use crate::app_error::AppError;
use crate::logic::AuthError;
use anyhow::bail;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHasher, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::PgPool;
use validator::{validate_email, HasLen};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldValidationError {
    EmailTaken,
    UsernameTaken,
    InvalidPasswordFmt,
    InvalidUsernameFmt,
    InvalidCaptchaID,
    InvalidCaptchaAnswer,
    InvalidEmailFmt,
    InvalidUrlToken,
    NotABee,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Email(String);
impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(email: String) -> Result<Self, FieldValidationError> {
        if validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(FieldValidationError::InvalidEmailFmt)
        }
    }

    pub async fn is_available(&self, pool: &PgPool) -> Result<bool, sqlx::Error> {
        let ret = sqlx::query!("select 1 as ret from users where email = $1", self.as_str())
            .fetch_optional(pool)
            .await?;

        Ok(ret.map(|_| false).unwrap_or(true))
    }
}

impl From<String> for Email {
    /// Should only be called to retrieve a username from postgres
    /// because it has already been parsed prior to insertion
    fn from(email_str: String) -> Self {
        Self(email_str)
    }
}

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Username(String);
impl Username {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(username: String) -> Result<Self, FieldValidationError> {
        let len = username.length();
        if len < 2 || len > 30 {
            Err(FieldValidationError::InvalidUsernameFmt)?;
        }

        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            Err(FieldValidationError::InvalidUsernameFmt)?;
        }

        Ok(Self(username))
    }

    pub async fn is_available(&self, pool: &PgPool) -> Result<bool, sqlx::Error> {
        let ret = sqlx::query!(
            "select 1 as ret from users where lower(username) = lower($1)",
            self.as_str(),
        )
        .fetch_optional(pool)
        .await?;

        Ok(ret.map(|_| false).unwrap_or(true))
    }
}

impl From<String> for Username {
    /// Should only be called to retrieve a username from postgres
    /// because it has already been parsed prior to insertion
    fn from(username_str: String) -> Self {
        Self(username_str)
    }
}

#[derive(Clone)]
pub struct Password(Secret<String>);
impl Password {
    pub fn expose_as_str(&self) -> &str {
        &self.0.expose_secret()
    }

    pub fn expose_as_bytes(&self) -> &[u8] {
        &self.0.expose_secret().as_bytes()
    }

    pub fn parse(password: Secret<String>) -> Result<Self, FieldValidationError> {
        let pwd = password.expose_secret();
        let len = pwd.len();
        if len < 8 || len > 100 {
            Err(FieldValidationError::InvalidPasswordFmt)?;
        }

        let mut valid = [false; 4];
        for c in pwd.chars() {
            if c.is_ascii_lowercase() {
                valid[0] = true;
            } else if c.is_ascii_uppercase() {
                valid[1] = true;
            } else if c.is_ascii_digit() {
                valid[2] = true;
            } else if c.is_ascii_punctuation() {
                valid[3] = true;
            }
        }

        if valid.contains(&false) {
            Err(FieldValidationError::InvalidPasswordFmt)?;
        }

        Ok(Self(password))
    }

    pub fn generate_argon2_hash(&self) -> anyhow::Result<PasswordHash> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        //todo: test failure (try hashing empty string)
        match argon2.hash_password(self.expose_as_bytes(), &salt) {
            Ok(hash) => Ok(PasswordHash::new(hash)),
            Err(e) => bail!(e),
        }
    }

    pub fn verify_password(&self, hash: &PasswordHash) -> Result<(), AppError> {
        let parsed_hash = match password_hash::PasswordHash::new(hash.expose_as_str()) {
            Ok(h) => h,
            Err(e) => {
                if e == password_hash::Error::Password {
                    Err(AuthError::InvalidPassword)?;
                }

                Err(AppError::with_msg(e.to_string()))?
            }
        };

        Argon2::default()
            .verify_password(self.expose_as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidPassword)?;

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct PasswordHash(Secret<String>);
impl PasswordHash {
    pub fn new(hash: argon2::PasswordHash) -> Self {
        Self {
            0: Secret::new(hash.to_string()),
        }
    }

    pub fn from_str(hash_str: String) -> Self {
        Self {
            0: Secret::new(hash_str),
        }
    }

    pub fn expose_as_str(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Serialize for PasswordHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expose_as_str())
    }
}

// Only used in tests
impl Default for PasswordHash {
    fn default() -> Self {
        Self {
            0: Secret::new("".into()),
        }
    }
}
