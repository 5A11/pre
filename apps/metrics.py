import contextlib

from prometheus_client import Counter, Gauge, Summary

PROM_NAMESPACE = "pre"

class NullSummary:
    def time(self):
        return contextlib.suppress()


class ProxyMetrics:
    def __init__(self, disable: bool = False) -> None:
        self._disable = disable
        if disable:
            return

        self._time_query_tasks = Summary(
            "proxy_time_to_query_tasks",
            "Measures the time for the proxy app to query next reencryption tasks",
            namespace=PROM_NAMESPACE,
        )
        self._time_process_task = Summary(
            "proxy_time_to_process_task",
            "Measures the time for the proxy app to process one reencryption request, including submission to contract",
            namespace=PROM_NAMESPACE,
        )

        self._tasks_success = Counter(
            "proxy_tasks_processed_successfully",
            "Counts the number of reencryption requests that the proxy app processed successfully",
            namespace=PROM_NAMESPACE,
        )
        self._tasks_failed = Counter(
            "proxy_tasks_failed_to_process",
            "Counts the number of reencryption requests that the proxy app failed to process",
            namespace=PROM_NAMESPACE,
        )
        self._tasks_pending = Gauge(
            "proxy_tasks_pending",
            "Counts the number of reencryption requests assigned for the proxy that are waiting for processing",
            namespace=PROM_NAMESPACE,
        )

        self._failures_contract_query = Counter(
            "proxy_failed_contract_query",
            "Counts the number of proxy contract query failures",
            namespace=PROM_NAMESPACE,
        )
        self._failures_contract_execution = Counter(
            "proxy_failed_contract_execution",
            "Counts the number of proxy contract execution failures",
            namespace=PROM_NAMESPACE,
        )
        self._failures_reencryption_umbral = Counter(
            "proxy_failed_umbral_reencryption",
            "Counts the number of proxy umbral reencryption failures",
            namespace=PROM_NAMESPACE,
        )

    @property
    def time_query_tasks(self):
        if self._disable:
            return NullSummary()
        return self._time_query_tasks

    @property
    def time_process_task(self):
        if self._disable:
            return NullSummary()
        return self._time_process_task

    def report_contract_query_failure(self):
        if self._disable:
            return
        self._failures_contract_query.inc()

    def report_contract_execution_failure(self):
        if self._disable:
            return
        self._failures_contract_execution.inc()

    def report_umbral_reencryption_failure(self):
        if self._disable:
            return
        self._failures_reencryption_umbral.inc()

    def report_task_succeeded(self):
        if self._disable:
            return
        self._tasks_success.inc()

    def report_task_failed(self):
        if self._disable:
            return
        self._tasks_failed.inc()

    def report_pending_tasks_count(self, count: int):
        if self._disable:
            return
        self._tasks_pending.set(count)
