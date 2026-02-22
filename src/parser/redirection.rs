#[derive(Debug, PartialEq)]
pub enum Descriptor {
    Stdin,
    Stdout,
    Stderr,
}
impl From<char> for Descriptor {
    fn from(s: char) -> Self {
        match s {
            '0' => Descriptor::Stdin,
            '1' => Descriptor::Stdout,
            '2' => Descriptor::Stderr,
            _ => panic!("Invalid descriptor: {}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    Input,
    Output,
}

#[derive(Debug, PartialEq)]
pub struct Redirection {
    pub descriptor: Descriptor,
    pub file: String,
    pub redirection_type: RedirectionType,
}
