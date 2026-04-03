use tracing::{debug, warn};
use uuid::Uuid;
use crate::{GNS3_TEMPLATE_PREFIX, GUEST_IMAGE_PATH};
use crate::models::gns3::connector::Gns3Connector;
use crate::models::gns3::template::{Gns3QemuTemplate, Gns3Template, TemplateCategory, TemplateType};
use crate::models::nodes::node::Node;
use crate::utils::files::shared_dir::SHARED_DIR_PATH;

const TARGET: &str = "template";

pub fn template_name(name: &str) -> String {
    format!("{} {}", GNS3_TEMPLATE_PREFIX.get().unwrap(), name)
}

pub fn find_and_delete_templates(gns3: &Gns3Connector) -> anyhow::Result<()> {
    let templates = gns3.get_templates()?;

    for template in templates {
        if template.name.starts_with(GNS3_TEMPLATE_PREFIX.get().unwrap()) {
            warn!(target: TARGET, "Deleting old template: {}", template.name);
            gns3.delete_template(&template.name)?;
        }
    }

    Ok(())
}

pub fn generate_and_create_guest_template(gns3: &Gns3Connector, guest_name: &str, node: &Node) -> anyhow::Result<()> {
    let template = Gns3Template {
        name: template_name(guest_name),
        category: TemplateCategory::Guest,
        builtin: false,
        template_id: Uuid::new_v4().to_string(),
        symbol: String::from("linux_guest.svg"),
        default_name_format: String::from("{name}-{0}"),
        compute_id: Some(String::from("local")),
        template_type: TemplateType::Qemu(Gns3QemuTemplate {
            adapter_type: String::from("virtio-net-pci"),
            adapters: 1,
            custom_adapters: vec![],
            cpus: node.vcpu,
            cpu_throttling: 0,
            ram: node.ram,
            legacy_networking: false,
            replicate_network_connection_state: true,
            mac_address: String::new(),
            console_type: String::from("telnet"),
            console_auto_start: false,
            create_config_disk: false,
            hda_disk_image: GUEST_IMAGE_PATH.get().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
            hda_disk_interface: String::from("scsi"),
            hdb_disk_image: String::new(),
            hdb_disk_interface: String::from("none"),
            hdc_disk_image: String::new(),
            hdc_disk_interface: String::from("none"),
            hdd_disk_image: String::new(),
            hdd_disk_interface: String::from("none"),
            first_port_name: String::new(),
            port_name_format: String::from("ens{port4}"),
            port_segment_size: 0,
            process_priority: String::from("normal"),
            qemu_path: String::from("qemu-system-x86_64"),
            platform: String::new(),
            on_close: String::from("power_off"),
            linked_clone: true,
            kernel_image: String::new(),
            kernel_command_line: String::new(),
            cdrom_image: String::new(),
            bios_image: String::new(),
            boot_priority: String::from("c"),
            initrd: String::new(),
            tpm: false,
            uefi: false,
            options: format!("-virtfs local,path=\"{}\",mount_tag=shared_folder,id=shared_folder,security_model=mapped-xattr", SHARED_DIR_PATH.display()),
            usage: String::from("Username:\troot\nPassword:\tdebian"),
        }),
    };

    gns3.create_template(&template)?;

    debug!(target: TARGET, "Generated guest template: {}", guest_name);

    Ok(())
}

pub fn generate_and_create_router_template(gns3: &Gns3Connector, router_name: &str, node: &Node, image_name: String) -> anyhow::Result<()> {
    let router = node.unwrap_router();

    let template = Gns3Template {
        name: template_name(router_name),
        category: TemplateCategory::Guest,
        builtin: false,
        template_id: Uuid::new_v4().to_string(),
        symbol: String::from(":/symbols/classic/router.svg"),
        default_name_format: String::from("{name}-{0}"),
        compute_id: Some(String::from("local")),
        template_type: TemplateType::Qemu(Gns3QemuTemplate {
            adapter_type: router.nics.values().into_iter().next().unwrap().nic_type.to_qemu_name(), // TODO: can be improved
            adapters: router.number_nics,
            custom_adapters: vec![],
            cpus: node.vcpu,
            cpu_throttling: 0,
            ram: node.ram,
            legacy_networking: false,
            replicate_network_connection_state: true,
            mac_address: String::new(),
            console_type: String::from("telnet"),
            console_auto_start: false,
            create_config_disk: false,
            hda_disk_image: image_name,
            hda_disk_interface: String::from("ide"),
            hdb_disk_image: String::new(),
            hdb_disk_interface: String::from("none"),
            hdc_disk_image: String::new(),
            hdc_disk_interface: String::from("none"),
            hdd_disk_image: String::new(),
            hdd_disk_interface: String::from("none"),
            first_port_name: String::new(),
            port_name_format: String::from("Ethernet{0}"),
            port_segment_size: 0,
            process_priority: String::from("normal"),
            qemu_path: String::from("qemu-system-x86_64"),
            platform: String::new(),
            on_close: String::from("power_off"),
            linked_clone: true,
            kernel_image: String::new(),
            kernel_command_line: String::new(),
            cdrom_image: String::new(),
            bios_image: String::new(),
            boot_priority: String::from("c"),
            initrd: String::new(),
            tpm: false,
            uefi: false,
            options: String::from("-mem-prealloc"),
            usage: String::new(),
        }),
    };

    gns3.create_template(&template)?;

    debug!(target: TARGET, "Generated router template: {}", router_name);

    Ok(())
}