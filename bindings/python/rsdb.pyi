"""Type stubs for rsdb — a fast, file-system based JSON database."""

from typing import Any

class RsDb:
    """A synchronous JSON file database.

    Example:
        >>> db = RsDb("db.json", {"posts": [], "users": []})
        >>> db.read()
        >>> data = db.get_data()
        >>> data["posts"].append({"id": 1, "title": "Hello"})
        >>> db.set_data(data)
        >>> db.write()
    """

    def __init__(self, path: str, default_data: Any) -> None:
        """Create a new database instance.

        Args:
            path: Path to the JSON file
            default_data: Default data used when file doesn't exist
        """
        ...

    def read(self) -> None:
        """Read data from the JSON file into memory."""
        ...

    def write(self) -> None:
        """Write the current in-memory data to the JSON file."""
        ...

    def get_data(self) -> Any:
        """Get the current data as a Python dict/list."""
        ...

    def set_data(self, data: Any) -> None:
        """Replace the in-memory data with a new object."""
        ...

    def update(self, patch: Any) -> None:
        """Apply a JSON merge patch (RFC 7396), then write to disk."""
        ...


def read_db(path: str, default_data: Any) -> Any:
    """Read a JSON database file and return its contents.

    Args:
        path: Path to the JSON file
        default_data: Default data if file doesn't exist
    """
    ...


def write_db(path: str, data: Any) -> None:
    """Write data to a JSON database file.

    Args:
        path: Path to the JSON file
        data: Data to write
    """
    ...
