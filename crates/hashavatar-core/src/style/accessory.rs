use crate::AvatarError;

/// Maximum number of requested or effective accessory layers.
pub const MAX_ACCESSORY_LAYERS: usize = 4;

/// Semantic placement slot for an accessory layer.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvatarAccessorySlot {
    /// Behind the subject.
    Back,
    /// Aura around or above the head.
    Aura,
    /// Hat, crown, or horns.
    Headwear,
    /// Ear-mounted accessory.
    Earwear,
    /// Face-mounted accessory other than eyewear.
    Facewear,
    /// Eye-mounted accessory.
    Eyewear,
    /// Neck-mounted accessory.
    Neckwear,
    /// Left-hand item; reserved until a built-in item is admitted.
    HandheldLeft,
    /// Right-hand item; reserved until a built-in item is admitted.
    HandheldRight,
    /// Foreground effect; reserved until a built-in effect is admitted.
    Foreground,
}

impl AvatarAccessorySlot {
    pub(crate) const ALL_ADMITTED: [Self; 6] = [
        Self::Aura,
        Self::Headwear,
        Self::Earwear,
        Self::Facewear,
        Self::Eyewear,
        Self::Neckwear,
    ];

    /// Returns the stable slot identifier.
    pub const fn catalog_id(self) -> u8 {
        self as u8
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Back => "back",
            Self::Aura => "aura",
            Self::Headwear => "headwear",
            Self::Earwear => "earwear",
            Self::Facewear => "facewear",
            Self::Eyewear => "eyewear",
            Self::Neckwear => "neckwear",
            Self::HandheldLeft => "handheld-left",
            Self::HandheldRight => "handheld-right",
            Self::Foreground => "foreground",
        }
    }

    pub(crate) const fn bit(self) -> u16 {
        1_u16 << self.catalog_id()
    }
}

/// Built-in accessory layer.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvatarAccessory {
    /// Glasses in the eyewear slot.
    Glasses = 1,
    /// Hat in the headwear slot.
    Hat = 2,
    /// Headphones in the earwear slot.
    Headphones = 3,
    /// Crown in the headwear slot.
    Crown = 4,
    /// Bowtie in the neckwear slot.
    Bowtie = 5,
    /// Eyepatch in the facewear slot.
    Eyepatch = 6,
    /// Scarf in the neckwear slot.
    Scarf = 7,
    /// Halo in the aura slot.
    Halo = 8,
    /// Horns in the headwear slot.
    Horns = 9,
}

impl AvatarAccessory {
    /// Complete admitted accessory catalog in frozen 1.x identifier order.
    pub const ALL: [Self; 9] = [
        Self::Glasses,
        Self::Hat,
        Self::Headphones,
        Self::Crown,
        Self::Bowtie,
        Self::Eyepatch,
        Self::Scarf,
        Self::Halo,
        Self::Horns,
    ];

    /// Selects an accessory deterministically from a trait sample.
    pub fn from_sample(value: u16) -> Self {
        Self::ALL
            .iter()
            .copied()
            .nth(usize::from(value) % Self::ALL.len())
            .unwrap_or(Self::Glasses)
    }

    /// Returns the stable 1.x catalog identifier.
    pub const fn catalog_id(self) -> u16 {
        self as u16
    }

    /// Returns the required semantic slot.
    pub const fn slot(self) -> AvatarAccessorySlot {
        match self {
            Self::Glasses => AvatarAccessorySlot::Eyewear,
            Self::Hat | Self::Crown | Self::Horns => AvatarAccessorySlot::Headwear,
            Self::Headphones => AvatarAccessorySlot::Earwear,
            Self::Bowtie | Self::Scarf => AvatarAccessorySlot::Neckwear,
            Self::Eyepatch => AvatarAccessorySlot::Facewear,
            Self::Halo => AvatarAccessorySlot::Aura,
        }
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Glasses => "glasses",
            Self::Hat => "hat",
            Self::Headphones => "headphones",
            Self::Crown => "crown",
            Self::Bowtie => "bowtie",
            Self::Eyepatch => "eyepatch",
            Self::Scarf => "scarf",
            Self::Halo => "halo",
            Self::Horns => "horns",
        }
    }
}

/// Fixed-capacity accessory request stack.
#[must_use = "pass the accessory stack to AvatarStyle"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessoryStack {
    entries: [AvatarAccessory; MAX_ACCESSORY_LAYERS],
    length: u8,
}

impl AccessoryStack {
    /// Creates an empty stack without allocation.
    pub const fn new() -> Self {
        Self {
            entries: [AvatarAccessory::Glasses; MAX_ACCESSORY_LAYERS],
            length: 0,
        }
    }

    /// Copies a caller slice into the bounded stack.
    pub fn from_slice(accessories: &[AvatarAccessory]) -> Result<Self, AvatarError> {
        let mut stack = Self::new();
        for accessory in accessories {
            stack.try_push(*accessory)?;
        }
        Ok(stack)
    }

    /// Appends one accessory or returns a typed capacity error.
    pub fn try_push(&mut self, accessory: AvatarAccessory) -> Result<(), AvatarError> {
        let index = usize::from(self.length);
        let slot = self
            .entries
            .get_mut(index)
            .ok_or(AvatarError::AccessoryCapacity {
                maximum: MAX_ACCESSORY_LAYERS,
            })?;
        *slot = accessory;
        self.length = self
            .length
            .checked_add(1)
            .ok_or(AvatarError::AccessoryCapacity {
                maximum: MAX_ACCESSORY_LAYERS,
            })?;
        Ok(())
    }

    /// Returns the number of requested accessories.
    pub const fn len(self) -> usize {
        self.length as usize
    }

    /// Returns whether the stack is empty.
    pub const fn is_empty(self) -> bool {
        self.length == 0
    }

    /// Iterates over the populated entries only.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = AvatarAccessory> + '_ {
        self.as_slice().iter().copied()
    }

    pub(crate) fn as_slice(&self) -> &[AvatarAccessory] {
        self.entries.get(..usize::from(self.length)).unwrap_or(&[])
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [AvatarAccessory] {
        let length = usize::from(self.length);
        self.entries.get_mut(..length).unwrap_or(&mut [])
    }
}

impl Default for AccessoryStack {
    fn default() -> Self {
        Self::new()
    }
}
