import contextlib

from prometheus_client import Counter, Gauge, Histogram


PROM_NAMESPACE = "pre"
PROM_INSTANCE_LABEL = "id"


class NullSummary:
    def time(self):
        return contextlib.suppress()


class ProxyMetrics:
    def __init__(self, label: str, disable: bool = False) -> None:
        self.label = label
        self._disable = disable
        if disable:
            return

        self._time_query_tasks = Histogram(
            "proxy_time_to_query_tasks",
            "Measures the time for the proxy app to query next reencryption tasks",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )
        self._time_process_task = Histogram(
            "proxy_time_to_process_task",
            "Measures the time for the proxy app to process one reencryption request, including submission to contract",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )

        self._tasks_success = Counter(
            "proxy_tasks_processed_successfully",
            "Counts the number of reencryption requests that the proxy app processed successfully",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )
        self._tasks_failed = Counter(
            "proxy_tasks_failed_to_process",
            "Counts the number of reencryption requests that the proxy app failed to process",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )
        self._tasks_pending = Gauge(
            "proxy_tasks_pending",
            "Counts the number of reencryption requests assigned for the proxy that are waiting for processing",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )

        self._failures_contract_query = Counter(
            "proxy_failed_contract_query",
            "Counts the number of proxy contract query failures",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )
        self._failures_contract_execution = Counter(
            "proxy_failed_contract_execution",
            "Counts the number of proxy contract execution failures",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )
        self._failures_reencryption_umbral = Counter(
            "proxy_failed_umbral_reencryption",
            "Counts the number of proxy umbral reencryption failures",
            namespace=PROM_NAMESPACE,
            labelnames=[PROM_INSTANCE_LABEL],
        )

    @property
    def time_query_tasks(self):
        if self._disable:
            return NullSummary()
        return self._time_query_tasks.labels(self.label)

    @property
    def time_process_task(self):
        if self._disable:
            return NullSummary()
        return self._time_process_task.labels(self.label)

    def report_contract_query_failure(self):
        if self._disable:
            return
        self._failures_contract_query.labels(self.label).inc()

    def report_contract_execution_failure(self):
        if self._disable:
            return
        self._failures_contract_execution.labels(self.label).inc()

    def report_umbral_reencryption_failure(self):
        if self._disable:
            return
        self._failures_reencryption_umbral.labels(self.label).inc()

    def report_task_succeeded(self):
        if self._disable:
            return
        self._tasks_success.labels(self.label).inc()

    def report_task_failed(self):
        if self._disable:
            return
        self._tasks_failed.labels(self.label).inc()

    def report_pending_tasks_count(self, count: int):
        if self._disable:
            return
        self._tasks_pending.labels(self.label).set(count)
