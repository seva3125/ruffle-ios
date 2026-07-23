# Unsigned IPA Workflow Design

## Goal

Fork `madsmtm/ruffle-ios` to `seva3125/ruffle-ios` and add a manually
triggered GitHub Actions workflow that produces an unsigned iPhone IPA for
installation through SideStore.

## Scope

The change adds build automation only. It does not alter application behavior,
repair the previously identified networking vulnerabilities, publish a GitHub
Release, or store Apple or Tailscale credentials.

## Workflow

The workflow will:

1. Run only when manually dispatched.
2. Use a GitHub-hosted macOS runner.
3. Check out the repository and locked dependencies.
4. Install the Java and Rust tooling required by the pinned Ruffle build.
5. add the `aarch64-apple-ios` Rust target.
6. Build the `ruffle-ios` Release configuration for a generic physical iOS
   device with code signing disabled.
7. Locate the resulting `ruffle-ios.app`, place it under `Payload/`, and create
   an unsigned `Ruffle-unsigned.ipa`.
8. Verify the IPA structure and application metadata.
9. Generate a SHA-256 checksum.
10. Upload the IPA and checksum as GitHub Actions artifacts with a limited
    retention period.

## Distribution

After the first successful workflow run, the artifact will be downloaded to
the local Mac and sent to the user's iPhone with Taildrop. GitHub Actions will
not join the user's Tailnet. SideStore is responsible for signing and
installing the transferred IPA.

## Failure Handling

The workflow will fail explicitly if the Xcode build fails, the expected
application bundle cannot be found, the IPA lacks `Payload/ruffle-ios.app`, or
the application metadata cannot be read. Build logs will remain available in
the GitHub Actions run.

## Validation

Before publication:

- validate the workflow syntax;
- inspect the exact repository diff;
- run any locally available workflow lint;
- push the branch and execute the workflow;
- require a successful GitHub Actions result;
- download and inspect the produced IPA and checksum;
- perform a final code and security audit of the workflow.

## Success Criteria

The fork contains the workflow, a manual run completes successfully, its
artifact contains an unsigned device-targeted `ruffle-ios.app`, the checksum
matches the downloaded IPA, and the IPA is transferred to the selected iPhone
through Taildrop.
