use crate::Event;
use uxn::{Ports, Uxn};
use zerocopy::{BigEndian, U16};

#[repr(C)]
#[derive(
    zerocopy::Immutable,
    zerocopy::IntoBytes,
    zerocopy::FromBytes,
    zerocopy::KnownLayout,
)]
pub struct TrackerPorts {
    vector: U16<BigEndian>,
    x: U16<BigEndian>,
    y: U16<BigEndian>,
    state: u8,
    _padding1: u8,
    _padding2: u16,
    scroll_x: U16<BigEndian>,
    scroll_y: U16<BigEndian>,
    _padding3: u16,
}

impl Ports for TrackerPorts {
    const BASE: u8 = 0xf0;
}

/// Stored tracker state
#[derive(Default)]
pub struct Tracker {
    /// Current position
    pos: (f32, f32),

    /// Accumulated scroll values, used for fractional scrolling
    scroll: (f32, f32),

    /// Bitfield of button state (bit 0: left, bit 1: middle, bit 2: right)
    buttons: u8,

    /// Set as true when a mouse DEI / DEO operator is called
    active: bool,
}

/// Update to tracker state
#[derive(Default, Debug)]
pub struct TrackerState {
    /// Current position
    pub pos: (f32, f32),

    /// Accumulated scroll values, used for fractional scrolling
    pub scroll: (f32, f32),

    /// Bitfield of button state (bit 0: left, bit 1: middle, bit 2: right)
    pub buttons: u8,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker::default()
    }

    /// Sets the active flag
    pub fn set_active(&mut self) {
        self.active = true
    }

    /// Checks whether the active flag has been set
    pub fn active(&self) -> bool {
        self.active
    }

    /// Updates the internal tracker state, pushing an event if it has changed
    pub fn update(
        &mut self,
        vm: &mut Uxn,
        state: TrackerState,
    ) -> Option<Event> {
        let mut changed = false;
        let m = vm.dev_mut::<TrackerPorts>();

        if state.pos != self.pos {
            m.x.set(state.pos.0 as u16);
            m.y.set(state.pos.1 as u16);
            changed = true;
            self.pos = state.pos;
        }

        self.scroll.0 += state.scroll.0 / 5.0;
        self.scroll.1 += state.scroll.1 / 5.0;

        // Send scrolls as one-tick updates on a per-frame basis
        if self.scroll.0.abs() > 1.0 {
            changed = true;
            let amount = self.scroll.0.abs().min(i16::MAX as f32)
                * self.scroll.0.signum();
            m.scroll_x.set((amount as i16) as u16);
            self.scroll.0 -= (amount as i16) as f32;
        } else {
            m.scroll_x.set(0);
        }

        if self.scroll.1.abs() > 1.0 {
            changed = true;
            let amount = self.scroll.1.abs().min(i16::MAX as f32)
                * self.scroll.1.signum();
            m.scroll_y.set((amount as i16) as u16);
            self.scroll.1 -= (amount as i16) as f32;
        } else {
            m.scroll_y.set(0);
        }

        if state.buttons != self.buttons {
            m.state = state.buttons;
            changed = true;
            self.buttons = state.buttons;
        }

        if changed {
            Some(Event {
                data: None,
                vector: m.vector.get(),
            })
        } else {
            None
        }
    }
}
