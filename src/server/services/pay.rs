use crate::server::{
    dtos::pay::{CustomSubscriptionRequest, PaymentIntent, PaymentIntentsRequest, RefundRequest, SubscriptionRequest},
    misc::error::{AppError, Res},
};
use log::{error, info};
use serde_json::json;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCustomer,
    CreateRefund, Currency, Customer, CustomerId, Event, EventObject, EventType, PaymentIntentId,
    Refund, Webhook,
};

pub fn create_client(secret_key: &str) -> Client {
    Client::new(secret_key)
}

pub async fn get_customer(client: &Client, customer_id: &str) -> Res<Customer> {
    let id = customer_id.parse::<CustomerId>().map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse customer id: {}. {}",
            customer_id, e
        ))
    })?;
    Customer::retrieve(&client, &id, &[])
        .await
        .map_err(AppError::from)
}

// pub async fn create_onetime_session(
//     client: &Client,
//     customer: &Customer,
//     req: OneTimeRequest,
// ) -> Res<CheckoutSession> {
//     let params = CreateCheckoutSession {
//         payment_method_types: Some(vec![stripe::CreateCheckoutSessionPaymentMethodTypes::Card]),
//         line_items: Some(vec![stripe::CreateCheckoutSessionLineItems {
//             price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
//                 currency: Currency::USD,
//                 product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
//                     name: "Test Product".to_string(),
//                     ..Default::default()
//                 }),
//                 unit_amount: Some(req.amount),
//                 ..Default::default()
//             }),
//             quantity: Some(1),
//             ..Default::default()
//         }]),
//         mode: Some(CheckoutSessionMode::Payment),
//         success_url: Some(req.success_url.as_str()),
//         cancel_url: Some(req.cancel_url.as_str()),
//         customer: Some(customer.id.clone()),
//         ..Default::default()
//     };
//     CheckoutSession::create(&client, params)
//         .await
//         .map_err(AppError::from)
// }

pub async fn create_subscription_session(
    client: &Client,
    customer: &Customer,
    req: SubscriptionRequest,
) -> Res<CheckoutSession> {
    let params = CreateCheckoutSession {
        payment_method_types: Some(vec![stripe::CreateCheckoutSessionPaymentMethodTypes::Card]),
        line_items: Some(vec![stripe::CreateCheckoutSessionLineItems {
            price: Some(req.price_id.to_string()),
            quantity: Some(1),
            ..Default::default()
        }]),
        mode: Some(CheckoutSessionMode::Subscription),
        success_url: Some(req.success_url.as_str()),
        cancel_url: Some(req.cancel_url.as_str()),
        customer: Some(customer.id.clone()),
        ..Default::default()
    };
    CheckoutSession::create(&client, params)
        .await
        .map_err(AppError::from)
}

pub async fn create_custom_subscription_session(
    client: &Client,
    customer: &Customer,
    req: CustomSubscriptionRequest,
) -> Res<CheckoutSession> {
    let recurring_opt = if let Some(info) = &req.recurring_info {
        info.into()
    } else {
        Err(AppError::BadRequest(
            "recurring_info is required".to_string(),
        ))?
    };

    let params = CreateCheckoutSession {
        payment_method_types: Some(vec![stripe::CreateCheckoutSessionPaymentMethodTypes::Card]),
        line_items: Some(vec![stripe::CreateCheckoutSessionLineItems {
            price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
                currency: Currency::USD,
                product: Some(req.product_id.clone()),
                recurring: recurring_opt,
                unit_amount: Some(req.amount),
                ..Default::default()
            }),
            quantity: Some(1),
            ..Default::default()
        }]),
        mode: Some(CheckoutSessionMode::Subscription),
        success_url: Some(req.success_url.as_str()),
        cancel_url: Some(req.cancel_url.as_str()),
        customer: Some(customer.id.clone()),
        ..Default::default()
    };
    CheckoutSession::create(&client, params)
        .await
        .map_err(AppError::from)
}

pub async fn create_customer(client: &Client, email: &str, name: &str) -> Res<Customer> {
    let params = CreateCustomer {
        email: Some(email),
        name: Some(name),
        ..Default::default()
    };

    Customer::create(client, params)
        .await
        .map_err(AppError::from)
}

pub fn construct_event(payload: &str, signature: &str, webhook_secret: &str) -> Res<Event> {
    match Webhook::construct_event(payload, signature, webhook_secret) {
        Ok(event) => Ok(event),
        Err(e) => {
            error!("Error constructing webhook event: {}", e);
            Err(AppError::BadRequest(format!("Webhook Error: {}", e)))
        }
    }
}

pub fn process_webhook_event(event: Event) -> Res<()> {
    info!("Processing webhook event: {}", event.type_);

    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            if let EventObject::PaymentIntent(payment_intent) = event.data.object {
                info!("PaymentIntent was successful: {}", payment_intent.id);
            }
        }
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = event.data.object {
                info!("Checkout session completed: {}", session.id);
            }
        }
        EventType::CustomerSubscriptionCreated => {
            if let EventObject::Subscription(subscription) = event.data.object {
                info!("Subscription created: {}", subscription.id);
            }
        }
        EventType::CustomerSubscriptionUpdated => {
            if let EventObject::Subscription(subscription) = event.data.object {
                info!("Subscription updated: {}", subscription.id);
            }
        }
        EventType::CustomerSubscriptionDeleted => {
            if let EventObject::Subscription(subscription) = event.data.object {
                info!("Subscription deleted: {}", subscription.id);
            }
        }
        _ => {
            info!("Unhandled event type: {}", event.type_);
        }
    }

    Ok(())
}

pub async fn process_refund(client: &Client, req: &RefundRequest) -> Res<Refund> {
    let mut params = CreateRefund::new();
    let payment_intent_id = req
        .payment_intent_id
        .parse::<PaymentIntentId>()
        .map_err(|e| {
            AppError::Internal(format!(
                "Failed to parse payment intent id: {}. {}",
                req.payment_intent_id, e
            ))
        })?;
    params.payment_intent = Some(payment_intent_id);

    if let Some(amount) = req.amount {
        params.amount = Some(amount);
    }

    if let Some(reason) = &req.reason {
        params.reason = match reason.as_str() {
            "duplicate" => Some(stripe::RefundReasonFilter::Duplicate),
            "fraudulent" => Some(stripe::RefundReasonFilter::Fraudulent),
            "requested_by_customer" => Some(stripe::RefundReasonFilter::RequestedByCustomer),
            _ => None,
        };
    }

    Refund::create(client, params).await.map_err(AppError::from)
}

pub async fn get_subscription_payment(
    client: &Client,
    subscription_id: &str,
    customer_id: &str,
) -> Res<serde_json::Value> {
    // Parse the subscription ID
    let sub_id = subscription_id
        .parse::<stripe::SubscriptionId>()
        .map_err(|e| AppError::BadRequest(format!("Invalid subscription ID: {}", e)))?;

    // Retrieve the subscription from Stripe
    let subscription = stripe::Subscription::retrieve(client, &sub_id, &[])
        .await
        .map_err(AppError::from)?;

    // Verify this subscription belongs to the authenticated user
    let cust_id = customer_id
        .parse::<CustomerId>()
        .map_err(|e| AppError::Internal(format!("Invalid customer ID: {}", e)))?;

    if !matches!(subscription.customer, stripe::Expandable::Id(id) if id == cust_id) {
        return Err(AppError::Forbidden(
            "You don't have permission to access this subscription".to_string(),
        ));
    }

    // Get the latest invoice for this subscription
    let invoices = stripe::Invoice::list(
        client,
        &stripe::ListInvoices {
            subscription: Some(sub_id.clone()),
            limit: Some(1),
            ..Default::default()
        },
    )
    .await
    .map_err(AppError::from)?;

    // Extract the payment intent ID from the invoice
    let payment_intent_id = invoices
        .data
        .first()
        .and_then(|invoice| invoice.payment_intent.as_ref())
        .map(|pi| pi.id().to_string());

    if let Some(payment_intent_id) = payment_intent_id {
        Ok(json!({
            "subscription_id": subscription_id,
            "payment_intent_id": payment_intent_id,
            "invoice_id": invoices.data.first().map(|inv| inv.id.to_string()),
            "amount_paid": invoices.data.first().map(|inv| inv.amount_paid),
            "currency": invoices.data.first().and_then(|inv| inv.currency.map(|c| c.to_string())),
        }))
    } else {
        Err(AppError::NotFound(
            "No payment information found for this subscription".to_string(),
        ))
    }
}

pub async fn get_customer_payment_intents(
    client: &Client,
    customer_id: &str,
    req: &PaymentIntentsRequest,
) -> Res<Vec<PaymentIntent>> {
    let cust_id = customer_id
        .parse::<CustomerId>()
        .map_err(|e| AppError::Internal(format!("Invalid customer ID: {}", e)))?;

    // Create list parameters with pagination options
    let mut params = stripe::ListPaymentIntents {
        customer: Some(cust_id),
        limit: req.limit.or(Some(25)), // Default to 25 if not specified
        ..Default::default()
    };

    // Add pagination cursors if provided
    if let Some(ref cursor) = req.ending_before {
        if let Ok(payment_intent_id) = cursor.parse::<stripe::PaymentIntentId>() {
            params.ending_before = Some(payment_intent_id);
        } else {
            return Err(AppError::BadRequest(
                "Invalid ending_before cursor".to_string(),
            ));
        }
    }

    if let Some(ref cursor) = req.starting_after {
        if let Ok(payment_intent_id) = cursor.parse::<stripe::PaymentIntentId>() {
            params.starting_after = Some(payment_intent_id);
        } else {
            return Err(AppError::BadRequest(
                "Invalid starting_after cursor".to_string(),
            ));
        }
    }

    // Call Stripe API to fetch payment intents
    let payment_intents = stripe::PaymentIntent::list(client, &params)
        .await
        .map_err(AppError::from)?;

    // Transform payment intents to JSON format with selected fields
    let payment_intents_json = payment_intents
        .data
        .into_iter()
        .map(|pi| {
            PaymentIntent {
                id: pi.id.to_string(),
                amount: pi.amount,
                currency: pi.currency.to_string(),
                status: pi.status.to_string(),
                created: pi.created,
            }
        })
        .collect();

    Ok(payment_intents_json)
}
