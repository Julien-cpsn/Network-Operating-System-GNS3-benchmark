use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use gns3fy_rs::{ConsoleType, Gns3Connector, Lookup, QemuTemplate, Template, TemplateKind};
use tracing::{debug, error, warn};
use crate::{GNS3_TEMPLATE_PREFIX, GUEST_IMAGE_PATH};
use crate::models::nodes::node::Node;
use crate::utils::files::shared_dir::SHARED_DIR_PATH;

const TARGET: &str = "template";

pub fn template_name(name: &str) -> String {
    format!("{} {}", GNS3_TEMPLATE_PREFIX.get().unwrap(), name)
}

pub async fn find_and_delete_templates(gns3: Arc<Gns3Connector>) -> anyhow::Result<()> {
    let templates = gns3.get_templates().await?;

    let mut template_deleted = false;

    for template in templates {
        if template.name.starts_with(GNS3_TEMPLATE_PREFIX.get().unwrap()) {
            warn!(target: TARGET, "Deleting old template: {}", template.name);
            gns3.delete_template(Lookup::Name(template.name.as_str())).await?;

            template_deleted = true;
        }
    }

    if template_deleted {
        debug!(target: TARGET, "No templates deleted");
    }

    Ok(())
}

pub async fn generate_and_create_guest_template(gns3: Arc<Gns3Connector>, guest_name: &str, node: &Node) -> anyhow::Result<String> {
    let qemu_template = QemuTemplate {
        qemu_path: Some(String::from("qemu-system-x86_64")),
        platform: None,
        ram: Some(node.ram as i64),
        cpus: Some(node.vcpu as i64),
        adapters: Some(1),
        adapter_type: Some(String::from("virtio-net-pci")),
        mac_address: None,
        first_port_name: None,
        port_name_format: Some(String::from("ens{port4}")),
        port_segment_size: Some(0),
        custom_adapters: None,
        options: Some(format!("-virtfs local,path=\"{}\",mount_tag=shared_folder,id=shared_folder,security_model=mapped-file", SHARED_DIR_PATH.display())),
        boot_priority: Some(String::from("c")),
        on_close: Some(String::from("power_off")),
        process_priority: Some(String::from("normal")),
        cpu_throttling: Some(0),
        legacy_networking: Some(false),
        linked_clone: Some(true),
        bios_image: None,
        cdrom_image: None,
        initrd: None,
        kernel_image: None,
        kernel_command_line: None,
        hda_disk_image: Some(GUEST_IMAGE_PATH.get().unwrap().file_name().unwrap().to_str().unwrap().to_string()),
        hda_disk_interface: Some(String::from("scsi")),
        hdb_disk_image: None,
        hdb_disk_interface: None,
        hdc_disk_image: None,
        hdc_disk_interface: None,
        hdd_disk_image: None,
        hdd_disk_interface: None,
    };

    let template_name = template_name(guest_name);
    let mut template = Template::new(gns3.clone(), &template_name, TemplateKind::Qemu(qemu_template))
        .with_compute_id("local")
        .with_category("guest")
        .with_symbol("linux_guest.svg");

    template.console_type = Some(ConsoleType::Telnet);
    template.console_auto_start = Some(false);
    template.default_name_format = Some(String::from("{name}-{0}"));
    template.builtin = Some(false);
    template.usage = Some(String::from("Username:\troot\nPassword:\tdebian"));

    if let Ok(Some(mut existing_template)) = gns3.get_template(Lookup::Name(template_name.as_str())).await {
        if existing_template != template {
            existing_template.delete().await?;
            template.create().await?;
            debug!(target: TARGET, "Generated guest template: {}", guest_name);
        }
    }
    else {
        debug!(target: TARGET, "Generated guest template: {}", guest_name);
        template.create().await?;
    }


    Ok(template.template_id.unwrap().to_string())
}

pub async fn generate_and_create_router_template(gns3: Arc<Gns3Connector>, router_name: &str, node: &Node, images_name: &Vec<PathBuf>) -> anyhow::Result<String> {
    let router = node.unwrap_router();

    let mut hda_image = None;
    let mut hdb_image = None;
    let mut hdc_image = None;
    let mut hdd_image = None;

    for (index, image_path) in images_name.iter().enumerate() {
        match index {
            0 => hda_image = Some(image_path.file_name().unwrap().to_str().unwrap().to_string()),
            1 => hdb_image = Some(image_path.file_name().unwrap().to_str().unwrap().to_string()),
            2 => hdc_image = Some(image_path.file_name().unwrap().to_str().unwrap().to_string()),
            3 => hdd_image = Some(image_path.file_name().unwrap().to_str().unwrap().to_string()),
            _ => {
                error!(target: TARGET, "{router_name}: QEMU supports only max 4 images per appliance");
                exit(1);
            }
        }
    }

    let qemu_template = QemuTemplate {
        qemu_path: Some(String::from("qemu-system-x86_64")),
        platform: None,
        ram: Some(node.ram as i64),
        cpus: Some(node.vcpu as i64),
        adapters: Some(router.number_nics as i64),
        adapter_type: Some(router.nics.values().into_iter().next().unwrap().nic_type.to_qemu_name()),
        mac_address: None,
        first_port_name: None,
        port_name_format: Some(String::from("Ethernet{0}")),
        port_segment_size: Some(0),
        custom_adapters: None,
        options: Some(String::from("-mem-prealloc -cpu host")),
        boot_priority: Some(String::from("c")),
        on_close: Some(String::from("power_off")),
        process_priority: Some(String::from("normal")),
        cpu_throttling: Some(0),
        legacy_networking: Some(false),
        linked_clone: Some(true),
        bios_image: None,
        cdrom_image: None,
        initrd: None,
        kernel_image: None,
        kernel_command_line: None,
        hda_disk_interface: Some(if hda_image.is_some() { String::from("ide") } else { String::from("none") }),
        hda_disk_image: hda_image,
        hdb_disk_interface: Some(if hdb_image.is_some() { String::from("ide") } else { String::from("none") }),
        hdb_disk_image: hdb_image,
        hdc_disk_interface: Some(if hdc_image.is_some() { String::from("ide") } else { String::from("none") }),
        hdc_disk_image: hdc_image,
        hdd_disk_interface: Some(if hdd_image.is_some() { String::from("ide") } else { String::from("none") }),
        hdd_disk_image: hdd_image,
    };

    let template_name = template_name(router_name);
    let mut template = Template::new(gns3.clone(), &template_name, TemplateKind::Qemu(qemu_template))
        .with_compute_id("local")
        .with_category("guest")
        .with_symbol(":/symbols/classic/router.svg");

    template.console_type = Some(ConsoleType::Telnet);
    template.console_auto_start = Some(false);
    template.default_name_format = Some(String::from("{name}-{0}"));
    template.builtin = Some(false);

    if let Ok(Some(mut existing_template)) = gns3.get_template(Lookup::Name(template_name.as_str())).await {
        if existing_template != template {
            existing_template.delete().await?;
            template.create().await?;
            debug!(target: TARGET, "Generated router template: {}", router_name);
        }
    }
    else {
        template.create().await?;
        debug!(target: TARGET, "Generated router template: {}", router_name);
    }

    Ok(template.template_id.unwrap().to_string())
}