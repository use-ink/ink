mod error;
mod new;

pub(crate) use self::{
    error::{
        CommandErrorKind,
        CommandError,
        Result,
    },
    new::execute_new,
};
