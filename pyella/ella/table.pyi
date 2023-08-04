import typing as T
from ella.types import DataType

class Publisher:
    def write(self, *args, **kwargs) -> None: ...
    def flush(self) -> None: ...
    def write_batch(self, *args, **kwargs) -> None: ...
    def close(self) -> None: ...
    def __enter__(self) -> "Publisher": ...
    def __exit__(self, exc_type, exc_value, traceback) -> None: ...

class Table:
    @property
    def id(self) -> str: ...
    def publish(self) -> Publisher: ...

class TableInfo: ...
class TopicInfo(TableInfo): ...

class TableAccessor:
    def get(self, table: str) -> T.Optional[Table]: ...
    def get_or_create(self, table: str, info: TableInfo) -> Table: ...
    def create(self, table: str, info: TableInfo) -> Table: ...
    def drop(self, table: str) -> None: ...
    def __getitem__(self, key: str) -> Table: ...

class ColumnInfo:
    @property
    def name(self) -> str: ...
    @property
    def dtype(self) -> DataType: ...
    @property
    def required(self) -> bool: ...
    @property
    def row_shape(self) -> T.Optional[T.List[int]]: ...
