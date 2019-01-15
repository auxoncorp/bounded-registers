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

register! {
    RNG,
    u32,
    RO,
    Working WIDTH(U1) OFFSET(U0),
    Width WIDTH(U2) OFFSET(U1) [
        Four = U0,
        Eight = U1,
        Sixteen = U2
    ],
    Data WIDTH(U16) OFFSET(U2)
}

/// Below is a tiny program which uses these three registers.  Because
/// the registers themselves, in particular the write-able ones,
/// require that interaction with them be total, you will find that
/// their use will imbue totality on the program.  Totality, in this
/// sense, is contagious. How this tends to present in Rust, is what
/// we've come to term being /beholden to the program counter/. When
/// there's program logic in the types, and we're statically checking
/// them, there's no other choice.
///
/// Because of this, we will have to write the narrative of our
/// program in a way that resides within the constraints of totality.
/// This means writing code that feels functional in paradigm and
/// eschews mutability.
///
/// Rust's substructural properties allow us to wrap the mutable
/// parts—the actual pointer where the register lives—in structures
/// which have the appearance of purity due to their "consumption" in
/// the linear-logic sense. This is illustrated in that, when we bang
/// on a register, its type changes. Its functions return a *new*
/// thing, with a new type, but under the covers simply mutates the
/// value at the pointer and yields the pointer to the new wrapper.
/// This results in functional-like immutability where we must thread
/// the "state" through the CFG, because the program's behavior /is/
/// its CFG, rather than just its control flow.
#[test]
fn fake_main() {
    // We begin by initializing our dependently-typed wrappers around
    // the register's pointers.
    // NB: In a real implementation, these would come from the address
    // present in the documentation.
    let status_ptr = &mut 0_u32 as *mut u32;
    let status_reg: Status::Register<U0> = Status::Register::new(status_ptr);

    let wait_ptr = &mut 0_u32 as *mut u32;
    let wait_reg: WaitForMe::Register<U0> = WaitForMe::Register::new(wait_ptr);

    let rng_ptr = &(4_u32 << 3) as *const u32;
    let rng_reg = RNG::Register::new(rng_ptr);

    // Next, we thread them through our program.
    set_status(rng_reg, status_reg, wait_reg);

    unsafe {
        println!("done.\n  vals: {} {} {}", *status_ptr, *wait_ptr, *rng_ptr)
    };
}

fn set_status(
    rng: RNG::Register,
    status: Status::Register<U0>,
    wait: WaitForMe::Register<U0>,
) {
    // An arbitrary case expression. This is to illustrate how
    // branching may look.
    let data = rng.read(RNG::Data::Read).unwrap();
    if data.val() > 4 {
        type On = Status::On::Field<U1>;
        type Blue = Status::Color::Blue::Type;
        let _: Status::Register<with!(On + Blue)> = status.modify();
        // In this arm of the case, we do something with the write
        // only register.
        update_wo(wait);
    } else {
        // otherwise we mark the thing as dead, and we're done.
        let _ = status.modify_field(Status::Dead::Field::<U1>::new());
    }
}

fn update_wo(wait: WaitForMe::Register<U0>) {
    type Please = WaitForMe::PleaseWait::Field<U1>;
    type Busy = WaitForMe::Reason::Busy::Type;
    let _: WaitForMe::Register<with!(Please + Busy)> = wait.modify();
}
