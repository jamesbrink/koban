//! Typed, resource-oriented access layered on top of the raw [`ApiClient`] JSON
//! methods.
//!
//! The generic methods ([`ApiClient::list_resource`], [`ApiClient::get_resource`],
//! ...) work with any type implementing the relevant serde traits, so callers can
//! use the built-in [models](crate::models) or their own. The accessor methods
//! ([`ApiClient::invoices`], [`ApiClient::clients`], ...) return a typed
//! [`Resources`] handle bound to a built-in model for the most ergonomic usage:
//!
//! ```no_run
//! # async fn run(client: &koban::ApiClient) -> koban::Result<()> {
//! let invoices = client.invoices().list().await?;
//! let invoice = client.invoices().get("abc123").await?;
//! # let _ = (invoices, invoice);
//! # Ok(())
//! # }
//! ```

use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    ApiClient, KobanError, Resource, Result,
    models::{
        Client, Credit, Data, Expense, Invoice, Paginated, Payment, Product, Project, Quote, Task,
        Vendor,
    },
};

impl ApiClient {
    /// Base API path for a resource family, for example `api/v1/invoices`.
    fn resource_base(resource: Resource) -> String {
        format!("api/v1/{}", resource.path())
    }

    /// List records of `resource`, returning the paginated `data`/`meta` envelope.
    pub async fn list_resource<T: DeserializeOwned>(
        &self,
        resource: Resource,
        query: &[(String, String)],
    ) -> Result<Paginated<T>> {
        let value = self.get_json(&Self::resource_base(resource), query).await?;
        decode(value)
    }

    /// Fetch a single record of `resource` by id.
    pub async fn get_resource<T: DeserializeOwned>(
        &self,
        resource: Resource,
        id: &str,
    ) -> Result<T> {
        let path = format!("{}/{id}", Self::resource_base(resource));
        let value = self.get_json(&path, &[]).await?;
        Ok(decode::<Data<T>>(value)?.data)
    }

    /// Create a record of `resource` from a serializable body.
    ///
    /// The body can be a built-in model, a caller-defined struct, or a
    /// [`serde_json::Value`] for partial payloads.
    pub async fn create_resource<T, B>(&self, resource: Resource, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let value = self
            .post_json(&Self::resource_base(resource), &[], &to_value(body)?)
            .await?;
        Ok(decode::<Data<T>>(value)?.data)
    }

    /// Update a record of `resource` by id from a serializable body.
    pub async fn update_resource<T, B>(&self, resource: Resource, id: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let path = format!("{}/{id}", Self::resource_base(resource));
        let value = self.put_json(&path, &[], &to_value(body)?).await?;
        Ok(decode::<Data<T>>(value)?.data)
    }

    /// Delete a record of `resource` by id, returning the deleted record.
    pub async fn delete_resource<T: DeserializeOwned>(
        &self,
        resource: Resource,
        id: &str,
    ) -> Result<T> {
        let path = format!("{}/{id}", Self::resource_base(resource));
        let value = self.delete_json(&path, &[]).await?;
        Ok(decode::<Data<T>>(value)?.data)
    }

    /// Typed handle for an arbitrary resource and caller-chosen model type.
    pub fn resource<T>(&self, resource: Resource) -> Resources<'_, T> {
        Resources::new(self, resource)
    }

    /// Typed handle for clients.
    pub fn clients(&self) -> Resources<'_, Client> {
        self.resource(Resource::Clients)
    }

    /// Typed handle for invoices.
    pub fn invoices(&self) -> Resources<'_, Invoice> {
        self.resource(Resource::Invoices)
    }

    /// Typed handle for payments.
    pub fn payments(&self) -> Resources<'_, Payment> {
        self.resource(Resource::Payments)
    }

    /// Typed handle for quotes.
    pub fn quotes(&self) -> Resources<'_, Quote> {
        self.resource(Resource::Quotes)
    }

    /// Typed handle for credits.
    pub fn credits(&self) -> Resources<'_, Credit> {
        self.resource(Resource::Credits)
    }

    /// Typed handle for products.
    pub fn products(&self) -> Resources<'_, Product> {
        self.resource(Resource::Products)
    }

    /// Typed handle for expenses.
    pub fn expenses(&self) -> Resources<'_, Expense> {
        self.resource(Resource::Expenses)
    }

    /// Typed handle for vendors.
    pub fn vendors(&self) -> Resources<'_, Vendor> {
        self.resource(Resource::Vendors)
    }

    /// Typed handle for projects.
    pub fn projects(&self) -> Resources<'_, Project> {
        self.resource(Resource::Projects)
    }

    /// Typed handle for tasks.
    pub fn tasks(&self) -> Resources<'_, Task> {
        self.resource(Resource::Tasks)
    }
}

/// A typed handle to a single Invoice Ninja resource family.
///
/// Obtained from [`ApiClient::resource`] or a resource accessor such as
/// [`ApiClient::invoices`]. The generic parameter `T` is the model type that
/// reads/writes deserialize into and serialize from.
pub struct Resources<'a, T> {
    client: &'a ApiClient,
    resource: Resource,
    _marker: PhantomData<fn() -> T>,
}

impl<'a, T> Resources<'a, T> {
    pub(crate) fn new(client: &'a ApiClient, resource: Resource) -> Self {
        Self {
            client,
            resource,
            _marker: PhantomData,
        }
    }

    /// The resource family this handle operates on.
    pub fn resource(&self) -> Resource {
        self.resource
    }
}

impl<T: DeserializeOwned> Resources<'_, T> {
    /// Fetch a single record by id.
    pub async fn get(&self, id: &str) -> Result<T> {
        self.client.get_resource(self.resource, id).await
    }

    /// List the first page of records.
    pub async fn list(&self) -> Result<Vec<T>> {
        Ok(self
            .client
            .list_resource::<T>(self.resource, &[])
            .await?
            .data)
    }

    /// List records with an explicit query, returning the paginated envelope.
    pub async fn list_paginated(&self, query: &[(String, String)]) -> Result<Paginated<T>> {
        self.client.list_resource(self.resource, query).await
    }

    /// Create a record from a serializable body.
    pub async fn create<B: Serialize + ?Sized>(&self, body: &B) -> Result<T> {
        self.client.create_resource(self.resource, body).await
    }

    /// Update a record by id from a serializable body.
    pub async fn update<B: Serialize + ?Sized>(&self, id: &str, body: &B) -> Result<T> {
        self.client.update_resource(self.resource, id, body).await
    }

    /// Delete a record by id, returning the deleted record.
    pub async fn delete(&self, id: &str) -> Result<T> {
        self.client.delete_resource(self.resource, id).await
    }
}

fn decode<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|source| KobanError::Decode {
        message: source.to_string(),
    })
}

fn to_value<B: Serialize + ?Sized>(body: &B) -> Result<Value> {
    serde_json::to_value(body).map_err(|source| KobanError::Decode {
        message: source.to_string(),
    })
}
