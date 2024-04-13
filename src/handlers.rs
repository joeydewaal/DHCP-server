use crate::{leases::LeaseRange, packet::{DHCPMessageType, Packet}};

#[derive(Debug)]
pub enum DiscoverResult {
    NoLeases,
}

pub fn on_dhcp_discover( mut packet: Packet, data: &mut LeaseRange ) -> Result<Packet, DiscoverResult> {
    println!("Got discover");

    println!("Client");
    packet.print();
    let ip = data.get_available_ip(packet.xid).ok_or(DiscoverResult::NoLeases)?;

    packet.yiaddr = ip;
    packet.into_response(DHCPMessageType::DHCPOFFER);
    packet.override_option(data.get_subnet_option());
    packet.override_option(data.get_leasetime_option());
    packet.override_option(data.get_server_id_option());

    println!("\nResponse");
    packet.print();
    Ok(packet)
}

pub fn on_dhcp_request( packet: Packet , data: &mut LeaseRange ) -> Result<Packet, ()> {
    println!("Got request");
    println!("Client");
    packet.print();

    let ip = packet.get_requested_ip().unwrap();

    let result = data.reserve_ip(packet, ip);
    println!("Result: {result:?}");
    Err(())
}

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }
