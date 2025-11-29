# mfaws

A command-line tool for managing AWS Multi-Factor Authentication (MFA) sessions. This tool simplifies the process of obtaining temporary session credentials for MFA-protected AWS accounts by automating the interaction with AWS CLI commands.

## Overview

mfaws streamlines AWS credential management by:

- Automatically discovering available AWS profiles configured on your system
- Listing MFA devices associated with the selected profile
- Requesting an MFA token from the user
- Obtaining temporary session credentials via AWS STS
- Storing the authorized credentials in your AWS credentials file

This eliminates the need to manually run multiple AWS CLI commands each time you need to authenticate with MFA.

## Requirements

- AWS CLI v2 (required for AWS API interactions)
- Configured AWS profiles in your `~/.aws/credentials` file
- MFA devices registered on your AWS account that uses a Token like authenticator

## Installation

### Build from Source

1. Build the project:
   ```bash
   cargo build --release
   ```

2. The compiled binary will be located at `target/release/mfaws`. You can move it to a location in your PATH for easy access:
   ```bash
   cp target/release/mfaws /usr/local/bin/
   ```

### Prerequisites Installation

Before using mfaws, ensure you have AWS CLI v2 installed. Follow the [AWS CLI v2 installation guide](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) for your operating system.

## Usage

Run the tool from your terminal:

```bash
mfaws
```

The tool will guide you through the following steps:

1. **Select a Profile**: Choose from the list of available AWS profiles configured in your credentials file.

2. **Select an MFA Device**: The tool will fetch and display all MFA devices associated with your selected profile.

3. **Enter MFA Token**: When prompted, enter the current 6-digit code from your MFA device.

4. **Automatic Credential Storage**: Upon successful authentication, mfaws will automatically create a new profile in your AWS credentials file with the format `<profile-name>_mfa-authorized`. This profile contains the temporary session credentials with their expiration time.

You can then use the generated MFA-authorized profile with AWS CLI commands:

```bash
aws s3 ls --profile <profile-name>_mfa-authorized
```

## How It Works

mfaws performs the following operations:

1. Reads your AWS credentials file to discover configured profiles
2. Calls AWS IAM to list MFA devices for the selected profile
3. Prompts you for your current MFA token code
4. Calls AWS STS `get-session-token` to obtain temporary credentials
5. Writes the temporary credentials to a new profile in your credentials file

The generated profile includes the session token required for authenticated AWS API calls when MFA is enforced.
