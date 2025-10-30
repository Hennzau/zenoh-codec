// A 1 byte storage marker used to store presence of optional fields or sizes for unbounded fields.
#[derive(Debug, PartialEq)]
pub struct Flag;

// A 1 byte storage marker used to store presence of optional fields and ids through bitmasking.
#[derive(Debug, PartialEq)]
pub struct Header;

#[derive(Debug, PartialEq)]
pub struct ExtBlockBegin;

#[derive(Debug, PartialEq)]
pub struct ExtBlockEnd;
