use itertools::{Itertools, Either};
use crate::models::{
    PayoutItem,
    Payout,
    PayoutStatus,
};

const NUM_APPROVALS_REQUIRED: usize = 2;


#[derive(Clone, Debug)]
pub struct PaidPayouts {
    pub approved_payouts: Vec<Payout>,
    pub refunding_payouts: Vec<Payout>,
    pub pending_payouts: Vec<Payout>,
    // ids
    pub approved_payout_ids: Vec<String>,
    pub refunding_payout_ids: Vec<String>,
    pub pending_payout_ids: Vec<String>,
}
impl PaidPayouts {
    pub fn new(
        approved_payouts: Vec<Payout>,
        refunding_payouts: Vec<Payout>,
        pending_payouts: Vec<Payout>,
    ) -> Self {

        let approved_payouts2 = approved_payouts.into_iter()
            .filter(|p: &Payout| p.payout_status == PayoutStatus::PROCESSING)
            .collect::<Vec<Payout>>();

        let approved_payout_ids = approved_payouts2.iter()
            .map(|p| p.id.clone())
            .collect::<Vec<String>>();

        let refunding_payouts2 = refunding_payouts.into_iter()
            .filter(|p: &Payout| p.payout_status == PayoutStatus::PENDING_REFUND)
            .collect::<Vec<Payout>>();

        let refunding_payout_ids = refunding_payouts2.iter()
            .map(|p| p.id.clone())
            .collect::<Vec<String>>();

        let pending_payouts2 = pending_payouts.into_iter()
            .filter(|p: &Payout| p.payout_status == PayoutStatus::PENDING_APPROVAL)
            .collect::<Vec<Payout>>();

        let pending_payout_ids = pending_payouts2.iter()
            .map(|p| p.id.clone())
            .collect::<Vec<String>>();

        Self {
            approved_payouts: approved_payouts2,
            refunding_payouts: refunding_payouts2,
            pending_payouts: pending_payouts2,
            // ids
            approved_payout_ids: approved_payout_ids,
            refunding_payout_ids: refunding_payout_ids,
            pending_payout_ids: pending_payout_ids,
        }
    }
}


#[derive(Clone, Debug)]
pub struct SignedPayouts {
    pub approved_payouts: Vec<Payout>,
    pub pending_payouts: Vec<Payout>,
}
impl SignedPayouts {
    pub fn new(
        payouts_pending_approval: Vec<Payout>,
        approver_id: &String
    ) -> Self {

        ////////////////////////////////////////////////////
        // 1. filter payouts not signed by this approver.
        let payouts_without_sig = payouts_pending_approval
            .into_iter()
            .filter(|p: &Payout| !p.approved_by_ids.contains(approver_id))
            .map(|p: Payout| p.append_approved_by_id(approver_id.to_string()))
            .collect::<Vec<Payout>>();


        ////////////////////////////////////////////////////
        ///// 2. Filter Payouts with enough approvals to payout
        let (approved_payouts, pending_payouts): (Vec<Payout>, Vec<Payout>) =
            payouts_without_sig
                .clone()
                .into_iter()
                .partition_map(|p: Payout| {
                    match p.approved_by_ids.len() >= NUM_APPROVALS_REQUIRED {
                        true => Either::Left(p),
                        false => Either::Right(p)
                    }
                });

        Self {
            // payouts_without_sig: payouts_without_sig,
            approved_payouts: approved_payouts,
            pending_payouts: pending_payouts,
        }
    }

    pub fn get_ids(&self, payout_approval_type: PayoutApprovalType) -> PayoutIds {

        let payouts = match payout_approval_type {
            PayoutApprovalType::Approved => &self.approved_payouts,
            PayoutApprovalType::Pending => &self.pending_payouts,
        };

        // 3a. payouts, get Ids
        let payouts_ids = payouts.iter()
            .map(|p: &Payout| p.id.clone())
            .collect::<Vec<String>>();

        // 3b. payouts, get PayoutItem Ids
        let payouts_pitem_ids = payouts.iter()
            .map(|p: &Payout| p.payout_item_ids.clone())
            .flatten()
            .collect::<Vec<String>>();

        // 3c. payouts, get approver Ids
        let payouts_approver_ids = match payouts.iter().next() {
            None => vec![],
            Some(payout) => payout.approved_by_ids.clone(),
        };

        PayoutIds {
            payout_ids: payouts_ids,
            pitem_ids: payouts_pitem_ids,
            approver_ids: payouts_approver_ids,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PayoutApprovalType {
    Approved,
    Pending
}

#[derive(Clone, Debug)]
pub struct PayoutIds {
    pub payout_ids: Vec<String>,
    pub pitem_ids: Vec<String>,
    pub approver_ids: Vec<String>,
}
