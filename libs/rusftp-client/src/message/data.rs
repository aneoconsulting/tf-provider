/*
SSH_FXP_DATA: 103
(DATA) | u32: id | u32: data length | u8[data length]: data |
 */

pub type Data = bytes::Bytes;
