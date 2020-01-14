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

use core::marker::PhantomData;

use crate::{
    env2::{
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        env_access::env_with,
        Env,
        EnvTypes,
        Result,
        Topics,
    },
    storage::{
        alloc::{
            Allocate,
            AllocateUsing,
            Initialize,
        },
        Flush,
        Key,
    },
};
use ink_prelude::vec::Vec;

#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Debug)]
/// A wrapper around environments to make accessing them more efficient.
pub struct EnvAccessMut<E> {
    /// The wrapped environment to access.
    env: PhantomData<E>,
}

impl<E> AllocateUsing for EnvAccessMut<E> {
    #[inline]
    unsafe fn allocate_using<A>(_alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self::default()
    }
}

impl<E> Flush for EnvAccessMut<E> {}

impl<E> Initialize for EnvAccessMut<E> {
    type Args = ();

    #[inline(always)]
    fn initialize(&mut self, _args: Self::Args) {}
}

impl<E> Default for EnvAccessMut<E> {
    fn default() -> Self {
        Self {
            env: Default::default(),
        }
    }
}

impl<T> EnvTypes for EnvAccessMut<T>
where
    T: EnvTypes,
{
    /// The type of an address.
    type AccountId = T::AccountId;
    /// The type of balances.
    type Balance = T::Balance;
    /// The type of hash.
    type Hash = T::Hash;
    /// The type of timestamps.
    type Moment = T::Moment;
    /// The type of block number.
    type BlockNumber = T::BlockNumber;
    /// The type of a call into the runtime
    type Call = T::Call;
}

macro_rules! impl_get_property_for {
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident< $prop_name:ident >() -> $ret:ty; $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub fn $fn_name(&mut self) -> $ret {
            env_with(|instance| {
                instance.$fn_name::<T>()
            })
        }

        impl_get_property_for!($($tt)*);
    };
    () => {}
}

/// Allow emitting generic events.
///
/// # Note
///
/// This trait is required in order to fix some name resolution orderings
/// in the ink! macro generated code.
pub trait EmitEvent<T>
where
    T: Env,
{
    /// Emits an event with the given event data.
    fn emit_event<Event>(&mut self, event: Event)
    where
        Event: Topics<T> + scale::Encode;
}

impl<T> EmitEvent<T> for EnvAccessMut<T>
where
    T: Env,
{
    /// Emits an event with the given event data.
    fn emit_event<Event>(&mut self, event: Event)
    where
        Event: Topics<T> + scale::Encode,
    {
        env_with(|instance| instance.emit_event::<T, _>(event))
    }
}

impl<T> EnvAccessMut<T>
where
    T: Env,
{
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
    pub fn set_rent_allowance(&mut self, new_value: T::Balance) {
        env_with(|instance| instance.set_rent_allowance::<T>(new_value))
    }

    /// Writes the value to the contract storage under the given key.
    pub fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        env_with(|instance| instance.set_contract_storage::<T, _>(key, value))
    }

    /// Returns the value stored under the given key in the contract's storage.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_contract_storage<R>(&mut self, key: Key) -> Result<R>
    where
        R: scale::Decode,
    {
        env_with(|instance| instance.get_contract_storage::<T, R>(key))
    }

    /// Clears the contract's storage key entry.
    pub fn clear_contract_storage(&mut self, key: Key) {
        env_with(|instance| instance.clear_contract_storage::<T>(key))
    }

    /// Invokes a contract message.
    ///
    /// # Errors
    ///
    /// If the called contract has trapped.
    pub fn invoke_contract(&mut self, call_data: &CallParams<T, ()>) -> Result<()> {
        env_with(|instance| instance.invoke_contract::<T>(call_data))
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
    pub fn eval_contract<R>(
        &mut self,
        call_data: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        R: scale::Decode,
    {
        env_with(|instance| instance.eval_contract::<T, R>(call_data))
    }

    /// Instantiates another contract.
    ///
    /// # Errors
    ///
    /// - If the instantiation process traps.
    /// - If the code hash is invalid.
    /// - If given too few endowment.
    /// - If the instantiation process runs out of gas.
    pub fn create_contract<C>(
        &mut self,
        params: &CreateParams<T, C>,
    ) -> Result<T::AccountId> {
        env_with(|instance| instance.create_contract::<T, C>(params))
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
    pub fn input(&mut self) -> CallData {
        env_with(|instance| instance.input::<T>())
    }

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// The setting of this property must be the last interaction between
    /// the executed contract and its environment.
    /// The environment access asserts this guarantee.
    pub fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        env_with(|instance| instance.output::<T, _>(return_value))
    }

    /// Returns a random hash.
    ///
    /// # Note
    ///
    /// The subject buffer can be used to further randomize the hash.
    pub fn random(&mut self, subject: &[u8]) -> T::Hash {
        env_with(|instance| instance.random::<T>(subject))
    }

    /// Prints the given contents to the environmental log.
    pub fn println(&mut self, content: &str) {
        env_with(|instance| instance.println::<T>(content))
    }

    /// Returns the value from the *runtime* storage at the position of the key.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_runtime_storage<R>(&mut self, key: &[u8]) -> Result<R>
    where
        R: scale::Decode,
    {
        env_with(|instance| instance.get_runtime_storage::<T, R>(key))
    }
}
