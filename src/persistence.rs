#![allow(unused)]
use axum_server::bind;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{NewPerson, Person};

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub async fn find_person(&self, id: Uuid) -> Result<Option<Person>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, name, nick, birth_date, stack
            FROM people
            WHERE id = $1
        ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub fn create_person(&self, new_person: NewPerson) -> Person {
        todo!()
    }

    pub fn search_people(&self, query: String) -> Vec<Person> {
        todo!()
    }

    pub fn count_people() -> u32 {
        todo!()
    }
}
