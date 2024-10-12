use crate::{error::DHCPError, packet::{DHCPMessageType, Packet}, state::DHCPState};

#[derive(Debug)]
pub enum DiscoverResult {
    NoLeases,
}

pub fn on_dhcp_discover( mut packet: Packet, state: DHCPState) -> Result<Packet, DHCPError> {
    let mut lease_range = state.lock();
    println!("Got discover");

    println!("Client");
    packet.print();
    let ip = lease_range.get_available_ip(packet.xid).unwrap();

    packet.yiaddr = ip;
    packet.into_response(DHCPMessageType::DHCPOFFER);
    packet.override_option(lease_range.get_subnet_option());
    packet.override_option(lease_range.get_leasetime_option());
    packet.override_option(lease_range.get_server_id_option());

    println!("\nResponse");
    packet.print();
    Ok(packet)
}

pub fn on_dhcp_request( packet: Packet , state: DHCPState) -> Result<Packet, DHCPError> {
    let mut lease_range = state.lock();
    println!("Got request");
    println!("Client");
    packet.print();

    let ip = packet.get_requested_ip().unwrap();

    let result = lease_range.reserve_ip(&packet, ip);
    println!("Result: {result:?}");
    Ok(packet)
}

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }
