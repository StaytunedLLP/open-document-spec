# Enterprise Compliance, Security Policies, & Onboarding Mega-Handbook

## Chapter 1: SOC2 Security Compliance Controls
Control CC6.1: Access controls are implemented for all production systems. Multi-factor authentication (MFA) is required.
Control CC6.8: Software vulnerability scanning is performed on all code repositories prior to production release.

## Chapter 2: HIPAA Data Privacy Rules
All Protected Health Information (PHI) must be encrypted at rest with AES-256 and in transit with TLS 1.3. Access to PHI tables is restricted to authorized medical personnel and audited via immutable log tables.

## Chapter 3: ISO 27001 Information Security
Security awareness training is mandatory every 6 months for all employees. Third-party vendor risk assessments must be renewed annually.

## Chapter 4: Onboarding Checklist for Developers
1. Request access to GitHub org.
2. Setup YubiKey MFA.
3. Complete security awareness video.
4. Clone main repo and run `./scripts/setup.sh`.
