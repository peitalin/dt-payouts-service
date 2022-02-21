
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub email: String,
    pub user_role: UserRole,
    #[serde(rename = "storeId")]
    pub store_id: Option<String>,
    #[serde(rename = "cartId")]
    pub cart_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
  /// Not logged in
  ANON,
  /// Logged in as somebody
  USER,
  /// A platform owner superuser (ie us)
  PLATFORM_ADMIN,
  /// The system / not a human
  SYSTEM,
}