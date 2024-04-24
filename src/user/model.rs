use std::error::Error;
use rustrict::{Censor, Type};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sea_orm::entity::prelude::*;

/// What am I?
/// A class meant to hold all the values the server uses to compute messages.
/// *Do not send me. Ever.*
#[derive(Clone)]
pub struct User {
    /// Why is the user id (the number after the @) not stored here?
    /// Because we can simplify this! Use the method `get_name_split()`.
    pub name: String,
    pub uuid: Uuid,
    pub glass: GlassModeration,
    pub sendable_user: SendableUser,
    // pub password: String,
    // pub email: String
}

impl User {
    pub fn new(name: String) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            name,
            uuid,
            glass: GlassModeration::default(),
            sendable_user: SendableUser::new(name, uuid)
        }
    }

    /// I exist because the name and id are merged into the name variable.
    /// I return them seperately!
    pub fn name_split(&self) -> (&str, &str) {
        self.name.as_str().rsplit_once('@').unwrap()
    }
}

/// What am I?
/// A struct so that we can save user data in the database.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment=1)]
    pub user_number: i32,
    #[sea_orm(column_name = "id", enum_name = "Id")]
    pub id: i32,
    #[sea_orm(column_name = "name", enum_name = "Name")]
    pub name: String,
    #[sea_orm(column_name = "uuid", enum_name = "UUID")]
    pub uuid: Uuid,
    #[sea_orm(column_name = "password", enum_name = "Password")]
    pub password: String,
    #[sea_orm(column_name = "email", enum_name = "Email")]
    pub email: String,
    #[sea_orm(column_name = "mod", enum_name = "Mod")]
    /// This is just the DB equivalent of `glass`.
    /// It's in JSON format.
    pub moderation_stats: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// What am I?
/// A stripped down version of the `User` struct so that you can send something to the other clients.
#[derive(Serialize, Deserialize, Clone)]
pub struct SendableUser {
    pub name: String,
    pub uuid: Uuid
}

impl SendableUser {
    pub fn new(name: String, uuid: Uuid) -> Self {
        Self {
            name,
            uuid
        }
    }
}

/// What am I?
/// A struct meant to hold all the values and functions for the cauto-mod/censoring of Arcs.
#[derive(Clone, Default)]
pub struct GlassModeration {
    reports: i32,
    warnings: i32,
    pub is_muted: bool
}

impl GlassModeration {
    /// Runs the given text through a censoring filter.
    /// This will add reports if it finds Type::OFFENSIVE, returning an error.
    /// If it finds no Type::OFFENSIVE, but Type::EVASIVE, it will instead warn the user.
    /// If the user is muted, it returns an error.
    pub fn process(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        if self.is_muted { return Err("User is muted".into()); }

        let (censored, analysis) = Censor::from_str(input)
            .with_censor_threshold(Type::SEVERE)
            .with_censor_first_character_threshold(Type::OFFENSIVE & Type::SEVERE)
            .with_ignore_false_positives(false)
            .with_ignore_self_censoring(false)
            .with_censor_replacement('*')
            .censor_and_analyze();

        if analysis.is(Type::OFFENSIVE & Type::SEVERE) {
            self.warn();
            Err("Message is inappropriate".into())
        } else {
            if analysis.is(Type::EVASIVE) {
                self.warn();
            }
            Ok(censored)
        }
    }

    /// Warns the user, adding a report if there are 5 warnings.
    pub fn warn(&mut self) {
        self.warnings += 1;
        if self.warnings >= 5 {
            self.warnings = 0;
            self.reports += 1;
        }
    }

    /// Reports the user, muting them if there are 10 warnings.
    pub fn report(&mut self) {
        self.reports += 1;
        if self.reports >= 10 {
            self.is_muted = true;
        }
    }

    /// Mutes the user.
    pub fn mute(&mut self) { self.is_muted = true; }

    /// Unmutes the user.
    pub fn unmute(&mut self) { self.is_muted = false; }
}