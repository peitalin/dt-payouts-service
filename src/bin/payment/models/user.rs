use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use gm::utils::typing;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserProfile {
    pub id: String,
    pub payment_methods: Option<Vec<String>>,
    pub default_payment_method: Option<String>,
    pub store_id: Option<String>,
    pub payout_method: Option<String>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStripeCustomerId {
    pub id: String,
    pub stripe_customer_id: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserId {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPublic {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
    pub store_id: Option<String>,
    pub payout_method: Option<String> // Paypal Payout Email
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRemovePaymentMethodResponse {
  pub user: UserPublic
}

#[test]
fn test_user_public_deserializes() {

    let test_str = r#"
    [
        {
            "email":"jade@gm.com",
            "username":null,
            "firstName":"Jade",
            "lastName":"P",
            "createdAt":"2019-10-23T05:17:03.944133",
            "storeId":"store_00332cef-5cb5-4ad3-a519-ada134bd15a1",
            "payoutMethod":"jade@paid.com"
        }
    ]
    "#;

    let res = serde_json::from_str::<Vec<UserPublic>>(test_str);
    match res {
        Ok(u) => {
            let profile: UserPublic = u.into_iter().next().unwrap();
            assert_eq!(
                profile.first_name.unwrap(),
                String::from("Jade")
            );
        },
        Err(e) => panic!(e.to_string()),
    };
}