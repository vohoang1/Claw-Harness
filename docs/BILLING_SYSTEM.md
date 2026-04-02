# Billing System with Stripe Integration

## Overview

Token-based billing system for Claw-Harness with Stripe integration for payment processing.

## Architecture

```
┌──────────────┐
│  Usage       │
│  Tracking    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Rating      │
│  Engine      │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Stripe      │
│  Invoicing   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Billing     │
│  Dashboard   │
└──────────────┘
```

## Token Pricing Tiers

```rust
#[derive(Debug, Clone)]
pub struct PricingTier {
    pub name: String,
    pub price_per_1k_tokens: f64,
    pub included_tokens: u64,
    pub monthly_fee: f64,
}

pub const PRICING_TIERS: &[PricingTier] = &[
    PricingTier {
        name: "free".to_string(),
        price_per_1k_tokens: 0.0,
        included_tokens: 10_000,
        monthly_fee: 0.0,
    },
    PricingTier {
        name: "pro".to_string(),
        price_per_1k_tokens: 0.002,
        included_tokens: 1_000_000,
        monthly_fee: 29.99,
    },
    PricingTier {
        name: "enterprise".to_string(),
        price_per_1k_tokens: 0.001,
        included_tokens: 10_000_000,
        monthly_fee: 299.99,
    },
];
```

## Usage Tracking

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub id: String,
    pub user_id: String,
    pub session_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub model: String,
    pub timestamp: DateTime<Utc>,
    pub cost_usd: f64,
}

pub struct UsageTracker {
    db: DbConnection,
    stripe: StripeClient,
}

impl UsageTracker {
    pub async fn track_usage(&self, usage: TokenUsage) -> anyhow::Result<()> {
        // Store in database
        self.db.insert("token_usage", &usage).await?;
        
        // Update user's monthly total
        self.update_monthly_total(&usage.user_id, usage.total_tokens).await?;
        
        // Check if over quota
        if self.is_over_quota(&usage.user_id).await? {
            self.send_over_quota_notification(&usage.user_id).await?;
        }
        
        Ok(())
    }

    pub async fn generate_invoice(
        &self,
        user_id: &str,
        period: &str,
    ) -> anyhow::Result<Invoice> {
        // Get usage for period
        let usage = self.db
            .query::<TokenUsage>(
                "SELECT * FROM token_usage WHERE user_id = $1 AND date_trunc('month', timestamp) = $2",
                &[&user_id, &period]
            )
            .await?;

        let total_tokens: u64 = usage.iter().map(|u| u.total_tokens).sum();
        let total_cost: f64 = usage.iter().map(|u| u.cost_usd).sum();

        // Get user's subscription
        let subscription = self.get_user_subscription(user_id).await?;

        // Calculate overage
        let overage_tokens = total_tokens.saturating_sub(subscription.included_tokens);
        let overage_cost = (overage_tokens as f64 / 1000.0) * subscription.price_per_1k_tokens;

        Ok(Invoice {
            user_id: user_id.to_string(),
            period: period.to_string(),
            subscription_fee: subscription.monthly_fee,
            usage_tokens: total_tokens,
            overage_tokens,
            overage_cost,
            total: subscription.monthly_fee + overage_cost,
            line_items: vec![
                LineItem {
                    description: format!("{} Plan", subscription.name),
                    amount: subscription.monthly_fee,
                },
                LineItem {
                    description: format!("Overage: {} tokens", overage_tokens),
                    amount: overage_cost,
                },
            ],
        })
    }
}
```

## Stripe Integration

```rust
use stripe::{Customer, Invoice, PaymentIntent, Subscription};

pub struct StripeClient {
    client: stripe::Client,
}

impl StripeClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: stripe::Client::new(api_key),
        }
    }

    pub async fn create_customer(
        &self,
        user_id: &str,
        email: &str,
    ) -> anyhow::Result<Customer> {
        stripe::Customer::create(
            &self.client,
            stripe::CreateCustomer {
                email: Some(email),
                metadata: Some(std::collections::HashMap::from([
                    ("claw_user_id", user_id),
                ])),
                ..Default::default()
            },
        )
        .await
        .map_err(Into::into)
    }

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
    ) -> anyhow::Result<Subscription> {
        stripe::Subscription::create(
            &self.client,
            stripe::CreateSubscription {
                customer: customer_id,
                items: Some(vec![stripe::SubscriptionItemParams {
                    price: Some(price_id),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await
        .map_err(Into::into)
    }

    pub async fn create_invoice(
        &self,
        customer_id: &str,
        amount: i64,
        description: &str,
    ) -> anyhow::Result<Invoice> {
        stripe::Invoice::create(
            &self.client,
            stripe::CreateInvoice {
                customer: customer_id,
                auto_advance: Some(true),
                ..Default::default()
            },
        )
        .await?;

        // Add line item
        stripe::InvoiceItem::create(
            &self.client,
            stripe::CreateInvoiceItem {
                customer: customer_id,
                amount: Some(amount),
                currency: Some(stripe::Currency::USD),
                description: Some(description),
                ..Default::default()
            },
        )
        .await?;

        // Finalize invoice
        stripe::Invoice::finalize(
            &self.client,
            &invoice_id,
            stripe::FinalizeInvoice::new(),
        )
        .await
        .map_err(Into::into)
    }

    pub async fn process_payment(
        &self,
        customer_id: &str,
        amount: i64,
    ) -> anyhow::Result<PaymentIntent> {
        stripe::PaymentIntent::create(
            &self.client,
            stripe::CreatePaymentIntent {
                customer: Some(customer_id),
                amount: Some(amount),
                currency: Some(stripe::Currency::USD),
                automatic_payment_methods: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(Into::into)
    }
}
```

## Billing Dashboard API

```rust
use axum::{Router, routing::get, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct BillingDashboard {
    current_period: String,
    tokens_used: u64,
    tokens_included: u64,
    tokens_remaining: u64,
    overage_tokens: u64,
    estimated_cost: f64,
    payment_method: Option<String>,
    invoices: Vec<InvoiceSummary>,
}

#[derive(Serialize)]
pub struct InvoiceSummary {
    id: String,
    date: String,
    amount: f64,
    status: String,
    pdf_url: String,
}

pub fn billing_router() -> Router {
    Router::new()
        .route("/api/v1/billing/dashboard", get(get_billing_dashboard))
        .route("/api/v1/billing/invoices", get(list_invoices))
        .route("/api/v1/billing/payment-method", post(update_payment_method))
        .route("/api/v1/billing/subscription", post(update_subscription))
}

async fn get_billing_dashboard(
    Extension(user): Extension<User>,
    State(state): Extension<AppState>,
) -> Json<BillingDashboard> {
    let usage = state.tracker.get_monthly_usage(&user.id).await;
    let subscription = state.stripe.get_subscription(&user.stripe_customer_id).await;
    let invoices = state.stripe.list_invoices(&user.stripe_customer_id).await;

    Json(BillingDashboard {
        current_period: usage.period,
        tokens_used: usage.total_tokens,
        tokens_included: subscription.included_tokens,
        tokens_remaining: subscription.included_tokens.saturating_sub(usage.total_tokens),
        overage_tokens: usage.total_tokens.saturating_sub(subscription.included_tokens),
        estimated_cost: usage.calculate_cost(&subscription),
        payment_method: subscription.payment_method,
        invoices: invoices.into_iter().map(|i| InvoiceSummary {
            id: i.id,
            date: i.created.format("%Y-%m-%d").to_string(),
            amount: i.total as f64 / 100.0,
            status: i.status,
            pdf_url: i.hosted_invoice_url,
        }).collect(),
    })
}
```

## Usage Alerts

```rust
pub struct AlertConfig {
    pub threshold_percent: u8,
    pub email_enabled: bool,
    pub webhook_enabled: bool,
}

impl UsageTracker {
    pub async fn check_alerts(&self, user_id: &str) -> anyhow::Result<()> {
        let usage = self.get_monthly_usage(user_id).await?;
        let subscription = self.get_subscription(user_id).await?;
        
        let usage_percent = (usage.total_tokens as f64 / subscription.included_tokens as f64 * 100.0) as u8;
        
        if usage_percent >= 80 {
            self.send_alert(user_id, AlertType::EightyPercent).await?;
        }
        
        if usage_percent >= 100 {
            self.send_alert(user_id, AlertType::OverQuota).await?;
        }
        
        Ok(())
    }

    async fn send_alert(&self, user_id: &str, alert_type: AlertType) -> anyhow::Result<()> {
        let user = self.get_user(user_id).await?;
        
        match alert_type {
            AlertType::EightyPercent => {
                self.email.send(
                    &user.email,
                    "80% of token quota used",
                    "You've used 80% of your monthly token quota. Consider upgrading your plan."
                ).await?;
            }
            AlertType::OverQuota => {
                self.email.send(
                    &user.email,
                    "Token quota exceeded",
                    "You've exceeded your monthly token quota. Additional charges will apply."
                ).await?;
            }
        }
        
        Ok(())
    }
}
```

## Prometheus Metrics

```rust
use prometheus::{IntCounterVec, GaugeVec, register_int_counter_vec, register_gauge_vec};

lazy_static! {
    pub static ref BILLING_INVOICE_TOTAL: IntCounterVec = register_int_counter_vec!(
        "claw_billing_invoice_total",
        "Total amount invoiced",
        &["user_id", "period"]
    ).unwrap();

    pub static ref BILLING_TOKENS_USED: GaugeVec = register_gauge_vec!(
        "claw_billing_tokens_used",
        "Tokens used in current period",
        &["user_id"]
    ).unwrap();

    pub static ref BILLING_OVERAGE_COST: GaugeVec = register_gauge_vec!(
        "claw_billing_overage_cost",
        "Overage cost in USD",
        &["user_id"]
    ).unwrap();
}
```

## Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: billing-service
  namespace: claw-system
spec:
  replicas: 2
  selector:
    matchLabels:
      app: billing
  template:
    metadata:
      labels:
        app: billing
    spec:
      containers:
      - name: billing
        image: claw-harness-billing:latest
        env:
        - name: STRIPE_API_KEY
          valueFrom:
            secretKeyRef:
              name: stripe-secrets
              key: api-key
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secrets
              key: url
        ports:
        - containerPort: 8080
```
