use std::ptr;

use crypto::md5::Md5;
use crypto::digest::Digest;

use convert::AsMutPtr;

pub fn digest(inputs: &[&[u8]], output: &mut [u8; 16]) {

    let mut digest = Md5::new();

    for input in inputs {
        digest.input(input);
    }

    digest.result(output);
}

pub fn digest_u32(inputs: &[&[u8]]) -> u32 {

    let mut output = [0; 16];
    digest(inputs, &mut output);

    [0,4,8,12].iter().fold(0, |a, &e| {
        let mut u: u32 = 0;
        unsafe {
            ptr::copy_nonoverlapping(&output[e], u.as_mut_ptr(), 4);
        }
        a ^ u
    })
}
