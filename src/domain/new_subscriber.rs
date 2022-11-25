use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl NewSubscriber {
    pub fn parse(name: String, email: String) -> Result<NewSubscriber, String> {
        let name = SubscriberName::parse(name)?;
        let email = SubscriberEmail::parse(email)?;

        Ok(NewSubscriber { name, email })
    }
}
