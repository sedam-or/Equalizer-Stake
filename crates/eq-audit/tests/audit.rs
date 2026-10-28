use anyhow::Result;
use eq_testkit::TestDb;
use sqlx::Error;

#[tokio::test]
async fn app_role_cannot_mutate() -> Result<()> {
    let db = TestDb::new().await;

    let role = sqlx::query_scalar::<_, String>("SELECT current_user")
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(role, "eq_app");

    let update = sqlx::query("UPDATE audit.entries SET after = '{}'::jsonb")
        .execute(&db.app_pool)
        .await;
    assert!(update.as_ref().is_err_and(is_permission_denied));

    let delete = sqlx::query("DELETE FROM audit.entries")
        .execute(&db.app_pool)
        .await;
    assert!(delete.as_ref().is_err_and(is_permission_denied));

    Ok(())
}

fn is_permission_denied(error: &Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("42501")
}
