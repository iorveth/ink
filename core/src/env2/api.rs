//! The public raw interface towards the host Wasm engine.
//!
//! # Note
//!
//! Prefer using the dedicated `EnvAccess` and `EnvAccessMut` types in order
//! to interoperate with the environment as they already have their associated
//! environemntal types associated to them and provide additional safety in some
//! scenarios.

use crate::{
    env2::{
        env_access::env_with,
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        Topics,
        Result,
        Env,
    },
    storage::Key,
};

macro_rules! impl_get_property_for {
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident< $prop_name:ident >() -> $ret:ty; $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub fn $fn_name<T>() -> $ret
        where
            T: Env,
        {
            env_with(|instance| instance.$fn_name::<T>())
        }

        impl_get_property_for!($($tt)*);
    };
    () => {}
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

/// Emits an event with the given event data.
pub fn emit_event<T, Event>(event: Event)
where
    T: Env,
    Event: Topics<T> + scale::Encode,
{
    env_with(|instance| instance.emit_event::<T, _>(event))
}

/// Sets the rent allowance of the executed contract to the new value.
pub fn set_rent_allowance<T>(new_value: T::Balance)
where
    T: Env,
{
    env_with(|instance| instance.set_rent_allowance::<T>(new_value))
}

/// Writes the value to the contract storage under the given key.
pub fn set_contract_storage<T, V>(key: Key, value: &V)
where
    T: Env,
    V: scale::Encode,
{
    env_with(|instance| instance.set_contract_storage::<T, V>(key, value))
}

/// Returns the value stored under the given key in the contract's storage.
///
/// # Errors
///
/// - If the key's entry is empty
/// - If the decoding of the typed value failed
pub fn get_contract_storage<T, R>(key: Key) -> Result<R>
where
    T: Env,
    R: scale::Decode,
{
    env_with(|instance| instance.get_contract_storage::<T, R>(key))
}

/// Clears the contract's storage key entry.
pub fn clear_contract_storage<T>(key: Key)
where
    T: Env,
{
    env_with(|instance| instance.clear_contract_storage::<T>(key))
}

/// Invokes a contract message.
///
/// # Errors
///
/// If the called contract has trapped.
pub fn invoke_contract<T>(call_data: &CallParams<T, ()>) -> Result<()>
where
    T: Env,
{
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
pub fn eval_contract<T, R>(call_data: &CallParams<T, ReturnType<R>>) -> Result<R>
where
    T: Env,
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
pub fn create_contract<T, C>(params: &CreateParams<T, C>) -> Result<T::AccountId>
where
    T: Env,
{
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
pub fn input<T>() -> CallData
where
    T: Env,
{
    env_with(|instance| instance.input::<T>())
}

/// Returns the value back to the caller of the executed contract.
///
/// # Note
///
/// The setting of this property must be the last interaction between
/// the executed contract and its environment.
/// The environment access asserts this guarantee.
pub fn output<T, R>(return_value: &R)
where
    T: Env,
    R: scale::Encode,
{
    env_with(|instance| instance.output::<T, R>(return_value))
}

/// Returns a random hash.
///
/// # Note
///
/// The subject buffer can be used to further randomize the hash.
pub fn random<T>(subject: &[u8]) -> T::Hash
where
    T: Env,
{
    env_with(|instance| instance.random::<T>(subject))
}

/// Prints the given contents to the environmental log.
pub fn println<T>(content: &str)
where
    T: Env,
{
    env_with(|instance| instance.println::<T>(content))
}

/// Returns the value from the *runtime* storage at the position of the key.
///
/// # Errors
///
/// - If the key's entry is empty
/// - If the decoding of the typed value failed
pub fn get_runtime_storage<T, R>(key: &[u8]) -> Result<R>
where
    T: Env,
    R: scale::Decode,
{
    env_with(|instance| instance.get_runtime_storage::<T, R>(key))
}
