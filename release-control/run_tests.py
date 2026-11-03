"""Run real unittest assertions and emit the existing native summary format."""
import sys
from pathlib import Path
import unittest


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.discover(str(Path(__file__).parent), pattern="test_*.py")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    failed = len(result.failures) + len(result.errors) + len(result.unexpectedSuccesses)
    skipped = len(result.skipped) + len(result.expectedFailures)
    print("{} pass".format(result.testsRun - failed - skipped))
    print("{} fail".format(failed))
    print("{} skip".format(skipped))
    raise SystemExit(0 if result.wasSuccessful() and not skipped else 1)
