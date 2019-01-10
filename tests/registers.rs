#[macro_use]
extern crate typenum;
#[macro_use]
extern crate registers;

use typenum::consts::{U0, U1};

use registers::read_write::With;

register! {
    Status,
    u8,
    RW,
    On WIDTH(U1) OFFSET(U0),
    Dead WIDTH(U1) OFFSET(U1),
    Color WIDTH(U3) OFFSET(U2) [
        Red = U1,
        Blue = U2,
        Green = U3,
        Yellow = U4
    ]
}

#[test]
fn test_reg_macro_rw() {
    let val = &mut 0_u32 as *mut u32;
    let reg = Status::Register::<U0>::new(val);
    let reg_prime = reg.modify_field(Status::Color::Blue::Term);
    assert_eq!(reg_prime.val(), 8_u32);
    assert_eq!(
        reg_prime.read(Status::Color::Read),
        Status::Color::Blue::Term
    );
}

register! {
    WaitForMe,
    u32,
    WO,
    PleaseWait WIDTH(U1) OFFSET(U0),
    Reason WIDTH(U2) OFFSET(U1) [
        Busy = U1,
        WaitingForSomethingElse = U2,
        ThingsAreHardRightNowOkay = U3
    ]
}

#[test]
fn test_reg_macro_wo() {
    // We set the initial value to 1.
    let mut ow_val = 0_u32;
    {
        // Then we give the register a pointer to this value, and
        // set the register's value as 0.
        let ow_val_ptr = &mut ow_val as *mut u32;
        let reg: WaitForMe::Register<U0> = WaitForMe::Register::new(ow_val_ptr);
        reg.modify_field(WaitForMe::Reason::ThingsAreHardRightNowOkay::Term);
    }

    // So when we read the value back out, it should match that of
    // the register's.
    assert_eq!(ow_val, 6);
}

register! {
    RNG,
    u32,
    RO,
    Working WIDTH(U1) OFFSET(U0),
    Width WIDTH(U2) OFFSET(U1) [
        Four = U0,
        Eight = U1,
        Sixteen = U2
    ]
}

#[test]
fn test_reg_macro_ro() {
    let ptr = &4_u32 as *const u32;
    let reg = RNG::Register::new(ptr);
    let width = reg.read(RNG::Width::Read).unwrap();
    assert_eq!(width.val(), RNG::Width::Values::Sixteen);
}

#[test]
fn test_with() {
    type On = Status::On::Field<U1>;
    type Dead = Status::Dead::Field<U1>;
    type Blue = Status::Color::Blue::Type;

    let val = &mut 0_u32 as *mut u32;
    let reg: Status::Register<U0> = Status::Register::new(val);
    let reg_prime: Status::Register<with!(On + Dead + Blue)> = reg.modify();
    assert_eq!(reg_prime.val(), 11_u32);
    assert_eq!(
        reg_prime.read(Status::Color::Read),
        Status::Color::Blue::Term
    );
}

#[test]
fn overwrite() {
    // We set the initial value to 1.
    let mut ow_val = 1_u32;
    {
        // Then we give the register a pointer to this value, and
        // set the register's value as 0.
        let ow_val_ptr = &mut ow_val as *mut u32;
        let _reg: Status::Register<U0> = Status::Register::new(ow_val_ptr);
    }

    // So when we read the value back out, it should match that of
    // the register's.
    assert_eq!(ow_val, 0);
}
