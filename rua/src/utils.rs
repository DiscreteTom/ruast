/// Take members from a struct.
///
/// `take(self, value);` will be translated to `let value = self.value;`.
#[macro_export]
macro_rules! take {
  ($self:ident, $($value:ident),*) => {
    $(
      let $value = $self.$value;
    )*
  };
}

/// Take mutable members from a struct.
///
/// `take_mut(self, value);` will be translated to `let mut value = self.value;`.
#[macro_export]
macro_rules! take_mut {
  ($self:ident, $($value:ident),*) => {
    $(
      let mut $value = $self.$value;
    )*
  };
}

/// Take option members from a struct.
///
/// `take_option(self, value);` will be translated to `let value = self.value.ok_or("missing value")?;`.
#[macro_export]
macro_rules! take_option {
  ($self:ident, $($option:ident),*) => {
    $(
      let $option = $self.$option.ok_or(format!("missing {}", stringify!($option)))?;
    )*
  };
}

/// Clone members from a struct.
///
/// `clone(self, value);` will be translated to `let value = self.value.clone();`.
#[macro_export]
macro_rules! clone {
  ($self:ident, $($member:ident),*) => {
    $(
      let $member = $self.$member.clone();
    )*
  };
}

/// Shorthand for `tokio::spawn`.
#[macro_export]
macro_rules! go {
  ($($body:stmt)*) => {
    tokio::spawn(async move {$($body)*});
  };
}
