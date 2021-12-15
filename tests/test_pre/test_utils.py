from pre.utils.loggers import default_logging_config, get_logger


def test_loggers():
    logger_test = get_logger("test")
    assert get_logger("test") == logger_test
    logger_test.warn("just a test")
    default_logger = get_logger("default")
