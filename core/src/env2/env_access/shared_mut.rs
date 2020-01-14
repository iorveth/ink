// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    env2::{
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        property,
        Env,
        GetProperty,
        Result,
        SetProperty,
        Topics,
    },
    storage::Key,
};
use ink_prelude::vec::Vec;

/// The single environmental instance.
static mut ENV_INSTANCE: EnvInstance = EnvInstance {
    buffer: Vec::new(),
    has_interacted: false,
    has_returned_value: false,
};

/// Executes the given closure on the environmental instance.
///
/// This is only safe in a Wasm environment that has no threading support.
pub fn env_with<F, R>(f: F) -> R
where
    F: FnOnce(&mut EnvInstance) -> R,
{
    unsafe { f(&mut ENV_INSTANCE) }
}

/// The actual environmental instance.
///
/// Has a buffer and some state variables in order to prevent superflous
/// allocation and prevent malicious operations.
pub struct EnvInstance {
    /// A buffer to make environment accesses
    /// more efficient by avoiding allocations.
    buffer: Vec<u8>,
    /// False as long as there has been no interaction between
    /// the executed contract and the environment.
    ///
    /// This flag is used to check at runtime if the environment
    /// is used correctly in respect to accessing its input.
    has_interacted: bool,
    /// True as long as the return value has not yet been set.
    ///
    /// This flag is used to check at runtime if the environment
    /// is used correctly in respect to returning its value.
    has_returned_value: bool,
}

/// Allow emitting generic events.
pub trait EmitEvent {
    /// Emits an event with the given event data.
    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: Env,
        Event: Topics<T> + scale::Encode;
}

impl EmitEvent for EnvInstance {
    /// Emits an event with the given event data.
    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: Env,
        Event: Topics<T> + scale::Encode,
    {
        <T as Env>::emit_event(&mut self.buffer, event)
    }
}

macro_rules! impl_get_property_for {
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident< $prop_name:ident >() -> $ret:ty; $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub fn $fn_name<T>(&mut self) -> $ret
        where
            T: Env,
        {
            self.assert_not_yet_returned();
            self.set_has_interacted();
            <T as GetProperty<property::$prop_name<T>>>::get_property(&mut self.buffer)
        }

        impl_get_property_for!($($tt)*);
    };
    () => {}
}

impl EnvInstance {
    /// Asserts that no value has been returned yet by the contract execution.
    fn assert_not_yet_returned(&self) {
        assert!(!self.has_returned_value)
    }

    /// Sets the flag for recording interaction between executed contract
    /// and environment to `true`.
    fn set_has_interacted(&mut self) {
        self.has_interacted = true;
    }

    impl_get_property_for! {
        /// Returns the address of the caller of the executed contract.
        fn caller<Caller>() -> T::AccountId;
        /// Returns the transferred balance for the contract execution.
        fn transferred_balance<TransferredBalance>() -> T::Balance;
        /// Returns the current price for gas.
        fn gas_price<GasPrice>() -> T::Balance;
        /// Returns the amount of gas left for the contract execution.
        fn gas_left<GasLeft>() -> T::Balance;
        /// Returns the current block time in milliseconds.
        fn now_in_ms<NowInMs>() -> T::Moment;
        /// Returns the address of the executed contract.
        fn address<Address>() -> T::AccountId;
        /// Returns the balance of the executed contract.
        fn balance<Balance>() -> T::Balance;
        /// Returns the current rent allowance for the executed contract.
        fn rent_allowance<RentAllowance>() -> T::Balance;
        /// Returns the current block number.
        fn block_number<BlockNumber>() -> T::BlockNumber;
        /// Returns the minimum balance of the executed contract.
        fn minimum_balance<MinimumBalance>() -> T::Balance;
    }

    /// Sets the rent allowance of the executed contract to the new value.
    pub fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: Env,
    {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        <T as SetProperty<property::RentAllowance<T>>>::set_property(
            &mut self.buffer,
            &new_value,
        )
    }

    /// Writes the value to the contract storage under the given key.
    pub fn set_contract_storage<T, V>(&mut self, key: Key, value: &V)
    where
        T: Env,
        V: scale::Encode,
    {
        <T as Env>::set_contract_storage(&mut self.buffer, key, value)
    }

    /// Returns the value stored under the given key in the contract's storage.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_contract_storage<T, R>(&mut self, key: Key) -> Result<R>
    where
        T: Env,
        R: scale::Decode,
    {
        <T as Env>::get_contract_storage(&mut self.buffer, key)
    }

    /// Clears the contract's storage key entry.
    pub fn clear_contract_storage<T>(&mut self, key: Key)
    where
        T: Env,
    {
        <T as Env>::clear_contract_storage(key)
    }

    /// Invokes a contract message.
    ///
    /// # Errors
    ///
    /// If the called contract has trapped.
    pub fn invoke_contract<T>(&mut self, call_data: &CallParams<T, ()>) -> Result<()>
    where
        T: Env,
    {
        <T as Env>::invoke_contract(&mut self.buffer, call_data)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Errors
    ///
    /// - If the called contract traps.
    /// - If the account ID is invalid.
    /// - If given too few endowment.
    /// - If arguments passed to the called contract are invalid.
    /// - If the called contract runs out of gas.
    pub fn eval_contract<T, R>(
        &mut self,
        call_data: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: Env,
        R: scale::Decode,
    {
        <T as Env>::eval_contract(&mut self.buffer, call_data)
    }

    /// Instantiates another contract.
    ///
    /// # Errors
    ///
    /// - If the instantiation process traps.
    /// - If the code hash is invalid.
    /// - If given too few endowment.
    /// - If the instantiation process runs out of gas.
    pub fn create_contract<T, C>(
        &mut self,
        params: &CreateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: Env,
    {
        <T as Env>::create_contract(&mut self.buffer, params)
    }

    /// Returns the input to the executed contract.
    ///
    /// # Note
    ///
    /// - The input is the 4-bytes selector followed by the arguments
    ///   of the called function in their SCALE encoded representation.
    /// - This property must be received as the first action an executed
    ///   contract to its environment and can only be queried once.
    ///   The environment access asserts this guarantee.
    pub fn input<T>(&mut self) -> CallData
    where
        T: Env,
    {
        assert!(!self.has_interacted);
        self.assert_not_yet_returned();
        self.set_has_interacted();
        <T as GetProperty<property::Input<T>>>::get_property(&mut self.buffer)
    }

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// The setting of this property must be the last interaction between
    /// the executed contract and its environment.
    /// The environment access asserts this guarantee.
    pub fn output<T, R>(&mut self, return_value: &R)
    where
        T: Env,
        R: scale::Encode,
    {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        self.has_returned_value = true;
        <T as Env>::output(&mut self.buffer, &return_value);
    }

    /// Returns a random hash.
    ///
    /// # Note
    ///
    /// The subject buffer can be used to further randomize the hash.
    pub fn random<T>(&mut self, subject: &[u8]) -> T::Hash
    where
        T: Env,
    {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        <T as Env>::random(&mut self.buffer, subject)
    }

    /// Prints the given contents to the environmental log.
    pub fn println<T>(&mut self, content: &str)
    where
        T: Env,
    {
        <T as Env>::println(content)
    }

    /// Returns the value from the *runtime* storage at the position of the key.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_runtime_storage<T, R>(&mut self, key: &[u8]) -> Result<R>
    where
        T: Env,
        R: scale::Decode,
    {
        T::get_runtime_storage(&mut self.buffer, key)
    }
}
