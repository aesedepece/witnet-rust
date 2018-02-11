//This file is part of Rust-Witnet.
//
//Rust-Witnet is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
//Rust-Witnet is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
// along with Rust-Witnet. If not, see <http://www.gnu.org/licenses/>.
//
//This file is based on p2p/src/msg.rs from
// <https://github.com/mimblewimble/grin>,
// originally developed by The Grin Developers and distributed under the
// Apache License, Version 2.0. You may obtain a copy of the License at
// <http://www.apache.org/licenses/LICENSE-2.0>.

//! Message types that transit over the network and related serialization code.

use num::FromPrimitive;

/// Types of messages
enum_from_primitive! {
	#[derive(Debug, Clone, Copy, PartialEq)]
	pub enum Type {
		Error,
		Hand,
		Shake,
		Ping,
		Pong,
		GetPeerAddrs,
		PeerAddrs,
		GetHeaders,
		Header,
		Headers,
		GetBlock,
		Block,
		GetCompactBlock,
		CompactBlock,
		Transaction,
	}
}