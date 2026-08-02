---
profile: guide
status: stable
description: Authentication types & JWT token validation rules
---

# User Authentication Guide

User authentication uses JWT tokens with RSA-256 signatures. Passwords are hashed using bcrypt with cost factor 12. Refresh tokens are stored in Redis with a 7-day TTL.
