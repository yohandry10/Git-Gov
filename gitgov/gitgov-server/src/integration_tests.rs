// Integration tests for GitGov Control Plane Server.
//
// These tests require a PostgreSQL database. Set TEST_DATABASE_URL to run them.
// The easiest way is to use docker-compose:
//
//   docker-compose up -d gitgov-db
//   TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov cargo test integration
//
// Tests that cannot connect to the DB are skipped (not failed).

#[macro_use]
mod common;
mod agent_governance;
mod agent_governance_attribution;
mod agent_governance_context;
mod agent_governance_dry_run;
mod alerts_exports_policy_requests;
mod basic_auth;
mod compliance_evidence_exports;
mod coverage_and_compliance;
mod events_and_admin;
mod first_governed_repo_setup;
mod org_invitations;
mod packet_reconstruction;
mod policy_enforcement;
mod release_binding;
mod tenant_workspace_scope;
