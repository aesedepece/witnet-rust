use actix::prelude::*;
use actix::{ActorFuture, Context, Handler, ResponseActFuture, WrapFuture};
use log;

use super::{InventoryManager, InventoryManagerError};
use crate::actors::messages::{
    AddItem, GetItem, GetItemBlock, GetItemTransaction, StoreInventoryItem,
};
use crate::storage_mngr;
use witnet_data_structures::chain::{
    Block, Hash, Hashable, InventoryEntry, InventoryItem, PointerToBlock,
};
use witnet_data_structures::transaction::Transaction;

////////////////////////////////////////////////////////////////////////////////////////
// ACTOR MESSAGE HANDLERS
////////////////////////////////////////////////////////////////////////////////////////

/// Handler for AddItem message
impl Handler<AddItem> for InventoryManager {
    type Result = ResponseActFuture<Self, (), InventoryManagerError>;

    fn handle(&mut self, msg: AddItem, _ctx: &mut Context<Self>) -> Self::Result {
        match msg.item {
            StoreInventoryItem::Block(block) => {
                let block_hash = block.hash();
                let mut key = match block_hash {
                    Hash::SHA256(h) => h.to_vec(),
                };
                // Add prefix to key to avoid confusing blocks with transactions
                key.insert(0, b'B');
                let fut = storage_mngr::put(&key, &block)
                    .into_actor(self)
                    .map_err(|e, _, _| {
                        log::error!("Couldn't persist block in storage: {}", e);
                        InventoryManagerError::MailBoxError(e)
                    })
                    .and_then(move |_, _, ctx| {
                        log::debug!("Successfully persisted block in storage");
                        // Store all the transactions as well
                        let items_to_add = block.txns.create_pointers_to_transactions(block_hash);

                        for (tx_hash, pointer_to_block) in items_to_add {
                            // TODO: is it a good idea to saturate the actor this way?
                            // TODO: implement AddItems
                            ctx.notify(AddItem {
                                item: StoreInventoryItem::Transaction(tx_hash, pointer_to_block),
                            });
                        }

                        fut::ok(())
                    });

                Box::new(fut)
            }
            StoreInventoryItem::Transaction(hash, pointer_to_block) => {
                log::info!("Saving transaction {}", hash);
                let mut key = match hash {
                    Hash::SHA256(h) => h.to_vec(),
                };
                // Add prefix to key to avoid confusing blocks with transactions
                key.insert(0, b'T');
                let fut = storage_mngr::put(&key, &pointer_to_block)
                    .into_actor(self)
                    .map_err(|e, _, _| {
                        log::error!("Couldn't persist transaction in storage: {}", e);
                        InventoryManagerError::MailBoxError(e)
                    })
                    .and_then(|_, _, _| {
                        log::debug!("Successfully persisted transaction in storage");
                        fut::ok(())
                    });

                Box::new(fut)
            }
        }
    }
}

/// Handler for GetItem message
impl Handler<GetItem> for InventoryManager {
    type Result = ResponseActFuture<Self, InventoryItem, InventoryManagerError>;

    fn handle(&mut self, msg: GetItem, ctx: &mut Context<Self>) -> Self::Result {
        let fut: Self::Result = match msg.item {
            InventoryEntry::Tx(hash) => Box::new(
                self.handle(GetItemTransaction { hash }, ctx)
                    .map(|tx, _, _| InventoryItem::Transaction(tx)),
            ),
            InventoryEntry::Block(hash) => Box::new(
                self.handle(GetItemBlock { hash }, ctx)
                    .map(|block, _, _| InventoryItem::Block(block)),
            ),
        };

        fut
    }
}

/// Handler for GetItem message
impl Handler<GetItemBlock> for InventoryManager {
    type Result = ResponseActFuture<Self, Block, InventoryManagerError>;

    fn handle(&mut self, msg: GetItemBlock, _ctx: &mut Context<Self>) -> Self::Result {
        let mut key_block = match msg.hash {
            Hash::SHA256(x) => x.to_vec(),
        };
        // Block prefix
        key_block.insert(0, b'B');

        let fut = storage_mngr::get::<_, Block>(&key_block)
            .into_actor(self)
            .then(move |res, _, _| match res {
                Ok(opt) => match opt {
                    None => fut::err(InventoryManagerError::ItemNotFound),
                    Some(block) => fut::ok(block),
                },
                Err(e) => {
                    log::error!("Couldn't get item from storage: {}", e);

                    fut::err(InventoryManagerError::MailBoxError(e))
                }
            });

        Box::new(fut)
    }
}

/// Handler for GetItem message
impl Handler<GetItemTransaction> for InventoryManager {
    type Result = ResponseActFuture<Self, Transaction, InventoryManagerError>;

    fn handle(&mut self, msg: GetItemTransaction, _ctx: &mut Context<Self>) -> Self::Result {
        let mut key_transaction = match msg.hash {
            Hash::SHA256(x) => x.to_vec(),
        };
        // First try to read block
        key_transaction.insert(0, b'T');

        let fut = storage_mngr::get::<_, PointerToBlock>(&key_transaction)
            .into_actor(self)
            .then(|res, act, ctx| match res {
                Ok(opt) => match opt {
                    None => {
                        let fut: Self::Result =
                            Box::new(fut::err(InventoryManagerError::ItemNotFound));
                        fut
                    }
                    Some(pointer_to_block) => {
                        // Recursion
                        let fut = act
                            .handle(
                                GetItemBlock {
                                    hash: pointer_to_block.block_hash,
                                },
                                ctx,
                            )
                            .then(move |res, _, _| {
                                match res {
                                    Ok(block) => {
                                        // Read transaction from block
                                        let tx = block.txns.get(pointer_to_block.transaction_index);
                                        match tx {
                                            Some(tx) if tx.hash() == msg.hash => fut::ok(tx),
                                            Some(_tx) => {
                                                // The transaction hash does not match
                                                fut::err(InventoryManagerError::NoTransactionInPointedBlock(pointer_to_block))
                                            }
                                            None => fut::err(
                                                InventoryManagerError::NoTransactionInPointedBlock(pointer_to_block),
                                            ),
                                        }
                                    }
                                    Err(InventoryManagerError::ItemNotFound) => {
                                        fut::err(InventoryManagerError::NoPointedBlock(pointer_to_block))
                                    }
                                    Err(e) => {
                                        log::error!("Couldn't get item from storage: {}", e);
                                        fut::err(e)
                                    }
                                }
                            });

                        Box::new(fut)
                    }
                },
                Err(e) => {
                    log::error!("Couldn't get item from storage: {}", e);
                    let fut: Self::Result =
                        Box::new(fut::err(InventoryManagerError::MailBoxError(e)));
                    fut
                }
            });

        Box::new(fut)
    }
}
