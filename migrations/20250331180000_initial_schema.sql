-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Represents the Organization whose assets are being tracked
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    username VARCHAR(150) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(50) DEFAULT 'ANALYST',
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_users_organization_id ON users(organization_id);

-- Core Asset table - represents anything discovered externally
CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    asset_type VARCHAR(50) NOT NULL, -- e.g., 'DOMAIN', 'IP_ADDRESS', 'WEB_APP', 'CERTIFICATE', 'CODE_REPO'
    value TEXT NOT NULL,             -- The actual asset identifier (e.g., 'example.com', '192.0.2.1', 'https://app.example.com')
    status VARCHAR(50) DEFAULT 'ACTIVE', -- e.g., 'ACTIVE', 'INACTIVE', 'ARCHIVED'
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- JSONB for flexible attributes specific to asset types or discovery sources
    attributes JSONB,
    -- Ensure uniqueness per organization for a given type and value
    UNIQUE (organization_id, asset_type, value)
);

-- Index for common lookups
CREATE INDEX idx_assets_organization_id ON assets(organization_id);
CREATE INDEX idx_assets_asset_type ON assets(asset_type);
CREATE INDEX idx_assets_last_seen ON assets(last_seen);

-- Represents open ports found on IP Address assets
CREATE TABLE ports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE, -- Foreign key to an asset of type 'IP_ADDRESS'
    port_number INT NOT NULL,
    protocol VARCHAR(10) NOT NULL, -- e.g., 'TCP', 'UDP'
    service_name VARCHAR(100),     -- e.g., 'http', 'ssh', 'unknown'
    banner TEXT,                   -- Service banner grabbed during scan
    status VARCHAR(50) DEFAULT 'OPEN', -- e.g., 'OPEN', 'CLOSED', 'FILTERED'
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (asset_id, port_number, protocol)
);

CREATE INDEX idx_ports_asset_id ON ports(asset_id);
CREATE INDEX idx_ports_last_seen ON ports(last_seen);

-- Represents technologies detected on assets (e.g., Nginx on an IP/Port, React on a WebApp)
CREATE TABLE technologies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,     -- e.g., 'Nginx', 'React', 'WordPress', 'OpenSSH'
    version VARCHAR(100),          -- Detected version (can be null)
    category VARCHAR(100),         -- e.g., 'Web Server', 'JavaScript Framework', 'CMS', 'OS'
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Consider adding unique constraint if appropriate (e.g., asset_id, name, version)
    UNIQUE (asset_id, name, version) -- Might be too strict if discovery methods differ
);

CREATE INDEX idx_technologies_asset_id ON technologies(asset_id);
CREATE INDEX idx_technologies_name ON technologies(name);
CREATE INDEX idx_technologies_last_seen ON technologies(last_seen);

-- Represents vulnerabilities or misconfigurations found on assets
CREATE TABLE vulnerabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    port_id UUID REFERENCES ports(id) ON DELETE SET NULL, -- Optional: Link directly to a specific port
    title VARCHAR(512) NOT NULL,      -- Short description, e.g., "Outdated OpenSSL version"
    description TEXT,                -- Detailed description
    severity VARCHAR(50) NOT NULL,    -- e.g., 'CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'INFO'
    status VARCHAR(50) DEFAULT 'OPEN', -- e.g., 'OPEN', 'CLOSED', 'ACCEPTED_RISK', 'FALSE_POSITIVE'
    cve_id VARCHAR(50),              -- CVE identifier if applicable (e.g., 'CVE-2021-44228')
    cvss_score NUMERIC(3, 1),        -- CVSS score if available
    evidence JSONB,                  -- Data supporting the finding
    remediation TEXT,                -- How to fix it
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- When was it last confirmed present
    resolved_at TIMESTAMPTZ,         -- When was it marked as not 'OPEN'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vulnerabilities_asset_id ON vulnerabilities(asset_id);
CREATE INDEX idx_vulnerabilities_severity ON vulnerabilities(severity);
CREATE INDEX idx_vulnerabilities_status ON vulnerabilities(status);
CREATE INDEX idx_vulnerabilities_cve_id ON vulnerabilities(cve_id);
CREATE INDEX idx_vulnerabilities_last_seen ON vulnerabilities(last_seen);

-- Represents discovery/scan jobs
CREATE TABLE discovery_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL, -- e.g., 'DNS_ENUM', 'PORT_SCAN', 'WEB_CRAWL', 'CERT_SCAN'
    status VARCHAR(50) NOT NULL,   -- e.g., 'PENDING', 'RUNNING', 'COMPLETED', 'FAILED'
    target TEXT,                   -- Optional: specific target for the job (e.g., a domain)
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    logs TEXT,                     -- Store logs or error messages related to the job
    configuration JSONB            -- Configuration used for this specific job run
);

CREATE INDEX idx_discovery_jobs_organization_id ON discovery_jobs(organization_id);
CREATE INDEX idx_discovery_jobs_status ON discovery_jobs(status);
CREATE INDEX idx_discovery_jobs_job_type ON discovery_jobs(job_type);

-- Link assets/findings back to the job that discovered/updated them
CREATE TABLE job_asset_links (
    job_id UUID NOT NULL REFERENCES discovery_jobs(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, asset_id)
);

