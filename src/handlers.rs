use crate::{leases::LeaseRange, packet::Packet};

pub fn on_dhcp_discover( packet: Packet, data: &mut LeaseRange ) -> Result<Packet, ()> {
    println!("Got discover");
    println!("offered ip: {:?}", data.get_available_ip(packet.xid));
    Err(())
}

pub fn on_dhcp_request( _packet: Packet , _data: &mut LeaseRange ) -> Result<Packet, ()> {
    println!("Got request");

    Err(())
}

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }

// pub fn on_dhcp_discover( packet: Packet,data: &mut LeaseRange ) -> Result<(), ()> {

//     Ok(())
// }
