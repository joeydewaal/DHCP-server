#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    BOOTREQUEST = 1,
    BOOTREPLY = 2,
}

impl TryFrom<u8> for MessageType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => MessageType::BOOTREQUEST,
            2 => MessageType::BOOTREPLY,
            _ => return Err(()),
        })
    }
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::BOOTREQUEST => 1,
            MessageType::BOOTREPLY => 2
        }
    }
}
