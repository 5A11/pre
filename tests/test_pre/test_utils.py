from pre.utils.loggers import get_logger


def test_loggers():
    logger_test = get_logger("test")
    assert get_logger("test") == logger_test
    logger_test.warn("just a test")
    default_logger = get_logger("default")
    assert default_logger
