use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ErrorAction, GeneralError, ToastInfo, ToastKind};

#[derive(Debug, Error, Serialize, Deserialize, From, Clone, PartialEq)]
pub enum RegisterError {
    #[error("Name already in use")]
    NameInUse,
    #[error(transparent)]
    General(GeneralError),
}

impl ErrorAction for RegisterError {
    fn should_logout(&self) -> bool {
        match self {
            RegisterError::NameInUse => false,
            RegisterError::General(general_error) => general_error.should_logout(),
        }
    }

    fn should_log(&self) -> bool {
        match self {
            RegisterError::NameInUse => false,
            RegisterError::General(general_error) => general_error.should_log(),
        }
    }

    fn toast_info(&self) -> Option<ToastInfo> {
        match self {
            RegisterError::NameInUse => Some(ToastInfo::new(
                "Registration failed",
                "name already in use",
                ToastKind::Error,
            )),
            RegisterError::General(general_error) => general_error.toast_info(),
        }
    }

    fn is_bug(&self) -> bool {
        match self {
            RegisterError::NameInUse => false,
            RegisterError::General(general_error) => general_error.is_bug(),
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, From, Clone, PartialEq)]
pub enum LoginError {
    #[error("Invalid name and/or password")]
    InvalidCredentials,
    #[error(transparent)]
    General(GeneralError),
}

impl ErrorAction for LoginError {
    fn should_logout(&self) -> bool {
        match self {
            LoginError::InvalidCredentials => false, // 'cause there no login in the first place
            LoginError::General(general_error) => general_error.should_logout(),
        }
    }

    fn should_log(&self) -> bool {
        match self {
            LoginError::InvalidCredentials => false,
            LoginError::General(general_error) => general_error.should_log(),
        }
    }

    fn toast_info(&self) -> Option<ToastInfo> {
        match self {
            LoginError::InvalidCredentials => Some(ToastInfo::new(
                "Failed to log in",
                "invalid name and/or password",
                ToastKind::Error,
            )),
            LoginError::General(general_error) => general_error.toast_info(),
        }
    }

    fn is_bug(&self) -> bool {
        match self {
            LoginError::InvalidCredentials => false,
            LoginError::General(general_error) => general_error.is_bug(),
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, From, Clone, PartialEq)]
pub enum GameError {
    #[error("You are not playing a quest now")]
    NoActiveQuest,
    #[error("You already play a quest")]
    AlreadyActiveQuest,
    #[error("Page out of order")]
    PageOutOfOrder,
    #[error(transparent)]
    General(GeneralError),
}

impl ErrorAction for GameError {
    fn should_logout(&self) -> bool {
        match self {
            GameError::NoActiveQuest
            | GameError::AlreadyActiveQuest
            | GameError::PageOutOfOrder => false,
            GameError::General(general_error) => general_error.should_logout(),
        }
    }

    fn should_log(&self) -> bool {
        match self {
            GameError::NoActiveQuest
            | GameError::AlreadyActiveQuest
            | GameError::PageOutOfOrder => true,
            GameError::General(general_error) => general_error.should_log(),
        }
    }

    fn toast_info(&self) -> Option<ToastInfo> {
        match self {
            GameError::NoActiveQuest
            | GameError::AlreadyActiveQuest
            | GameError::PageOutOfOrder => None,
            GameError::General(general_error) => general_error.toast_info(),
        }
    }

    fn is_bug(&self) -> bool {
        match self {
            GameError::NoActiveQuest
            | GameError::AlreadyActiveQuest
            | GameError::PageOutOfOrder => true,
            GameError::General(general_error) => general_error.is_bug(),
        }
    }
}
