#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	// 常用的宏
	use frame_support::{pallet_prelude::*, traits::Hooks};
	// 常用工具方法
	use frame_system::{
		ensure_signed,
		pallet_prelude::{BlockNumberFor, *},
	};
	// 引入数据类型
	use sp_std::prelude::*;

	// 通过继承拥有了 frame_system::Config 里定义的数据类型
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// pallet::constant 用于声明这是个链上的常量
		#[pallet::constant]
		/// The maximum length of claim that can be added.
		type MaxClaimLength: Get<u32>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	// 因为我们要定义存储项，所以需要 pallet::generate_store 宏
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// 定义存储项
	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		// 密码安全的hash算法
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	// 定义事件
	#[pallet::event]
	// 生成工具函数
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, T::AccountId, Vec<u8>),
	}

	// 定义错误
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	// 用于定义回调函数，在区块的不同时期执行
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// 定义可调用函数
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 验证签名
			let sender = ensure_signed(origin)?;

			// 尝试从 Vec<u8> 转换为 BoundedVec<u8, T::MaxClaimLength>
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 验证是否已经存储过
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
				&bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 验证签名
			let sender = ensure_signed(origin)?;

			// 尝试从 Vec<u8> 转换为 BoundedVec<u8, T::MaxClaimLength>
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 校验是否已经存在存证
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 验证存证的所有者是否是当前用户
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 从存储里删除存证
			Proofs::<T>::remove(&bounded_claim);

			// 发送存证吊销事件
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// 验证签名
			let sender = ensure_signed(origin)?;

			// 尝试从 Vec<u8> 转换为 BoundedVec<u8, T::MaxClaimLength>
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 校验是否已经存在存证
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 验证存证的所有者是否是当前用户
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 从存储里删除存证
			Proofs::<T>::insert(&bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));

			// 发送存证转移事件
			Self::deposit_event(Event::ClaimTransfered(owner, sender, claim));

			Ok(().into())
		}
	}
}
