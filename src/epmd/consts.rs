// -- ~/lib/erl_interface/src/epmd/ei_epmd.h --

pub const EPMD_PORT           :u16 = 4369;

pub const EI_DIST_HIGH        :u16 = 5;
pub const EI_DIST_LOW         :u16 = 1;
pub const EI_MYPROTO           :u8 = 0;

pub const EI_EPMD_ALIVE2_REQ   :u8 = 0x78; // 120: 'x'
pub const EI_EPMD_ALIVE2_RESP  :u8 = 0x79; // 121: 'y'
pub const EI_EPMD_PORT2_REQ    :u8 = 0x7a; // 122: 'z'
pub const EI_EPMD_PORT2_RESP   :u8 = 0x77; // 119: 'w'

// -- --

pub const EI_HIDDEN_NODE       :u8 = 0x68; // 104: 'h'
pub const EI_SUCCESS           :i8 = 0x00;
