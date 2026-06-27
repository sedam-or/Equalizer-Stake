FROM python:3.12-slim AS builder

WORKDIR /app

# Install system deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY pyproject.toml .
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir --prefix=/install .

FROM python:3.12-slim AS runtime

# Non-root user
RUN groupadd -r eqs && useradd -r -g eqs -d /app -s /sbin/nologin eqs

WORKDIR /app

# Copy installed deps
COPY --from=builder /install /usr/local

# Copy app code
COPY . .

# Create media directory
RUN mkdir -p /app/media && chown -R eqs:eqs /app

USER eqs

EXPOSE 8000

CMD ["python", "-m", "uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
