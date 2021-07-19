#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    pallet_prelude::*,
    // traits::Randomness,
    Parameter,
    RuntimeDebug,
    StorageDoubleMap,
    StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd, One};

// use sp_io::hashing::blake2_128;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Note(pub Vec<u8>);

//pub struct Note(pub [u8; 16]);

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type NoteIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy;
}

decl_event! {
    pub enum Event<T> where
    <T as frame_system::Config>::AccountId,
    <T as Config>::NoteIndex,
    {
        NoteCreated(AccountId, NoteIndex, Note),
        NoteTransferred(AccountId, AccountId, NoteIndex),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        NotesdIdOverflow,
        InvalidNoteId,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Notes {
        pub Notes get(fn notes): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::NoteIndex => Option<Note>;
        // Store next note ID
        pub NextNoteId get(fn next_note_id): T::NoteIndex;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 1000]
        pub fn create(origin, ipfs_cid: Vec<u8>) {
            let sender = ensure_signed(origin)?;
            // let static_ipfs = "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco";
            // let note_hash = static_ipfs.using_encoded(blake2_128);
            //let _note = Note(static_ipfs.as_bytes().to_vec());
            let note = Note(ipfs_cid);

            // let note_id = Self::next_note_id();
            let note_id = Self::get_next_note_id()?;

            Notes::<T>::insert(&sender, note_id, note.clone());
            // NextNoteId::put(note_id + 1);
            Self::deposit_event(RawEvent::NoteCreated(sender, note_id, note));
        }



        /// Transfer note to new owner
        #[weight = 1000]
        pub fn transfer(origin, to: T::AccountId, note_id: T::NoteIndex) {
            let sender = ensure_signed(origin)?;

            Notes::<T>::try_mutate_exists(sender.clone(), note_id, |note| -> DispatchResult {
                if sender == to {
                    ensure!(note.is_some(), Error::<T>::InvalidNoteId);
                    return Ok(());
                }

                let note = note.take().ok_or(Error::<T>::InvalidNoteId)?;

                Notes::<T>::insert(&to, note_id, note);

                Self::deposit_event(RawEvent::NoteTransferred(sender, to, note_id));

                Ok(())
            })?;
        }
    }
}

impl<T: Config> Module<T> {
    fn get_next_note_id() -> sp_std::result::Result<T::NoteIndex, DispatchError> {
        NextNoteId::<T>::try_mutate(
            |next_id| -> sp_std::result::Result<T::NoteIndex, DispatchError> {
                let current_id = *next_id;
                *next_id = next_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NotesdIdOverflow)?;
                Ok(current_id)
            },
        )
    }
}
