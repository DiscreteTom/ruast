extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse, DeriveInput};

#[proc_macro_derive(BasicPeer)]
pub fn peer_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl Peer for #name {
      fn id(&self) -> i32 {
        self.id
      }
      fn set_tag(&mut self, tag: String) {
        self.tag = tag;
      }
      fn tag(&self) -> &str {
        &self.tag
      }
      fn tx(&self) -> &Sender<PeerEvent> {
        &self.tx
      }
    }
  };

  gen.into()
}
