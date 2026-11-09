use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::error::Result;
use crate::http::{Operation, Request, Response};

pub(crate) struct Call<T> {
    request: Request,
    output: PhantomData<fn() -> T>,
}

impl<T> Call<T> {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            output: PhantomData,
        }
    }
}

impl<T: DeserializeOwned> Operation for Call<T> {
    type Output = T;

    fn request(&self) -> Result<Request> {
        Ok(self.request.clone())
    }

    fn parse(&self, response: Response) -> Result<T> {
        response.ensure_success()?.json()
    }
}

pub(crate) struct NoContent {
    request: Request,
}

impl NoContent {
    pub fn new(request: Request) -> Self {
        Self { request }
    }
}

impl Operation for NoContent {
    type Output = ();

    fn request(&self) -> Result<Request> {
        Ok(self.request.clone())
    }

    fn parse(&self, response: Response) -> Result<()> {
        response.ensure_success().map(drop)
    }
}

pub(crate) struct Download {
    request: Request,
}

impl Download {
    pub fn new(request: Request) -> Self {
        Self { request }
    }
}

impl Operation for Download {
    type Output = Vec<u8>;

    fn request(&self) -> Result<Request> {
        Ok(self.request.clone())
    }

    fn parse(&self, response: Response) -> Result<Vec<u8>> {
        Ok(response.ensure_success()?.body)
    }
}
