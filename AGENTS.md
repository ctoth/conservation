# Conservation execution changes

Extend DenseState and FlowTopology for native batch execution; do not add a
second settlement implementation. Keep lane and step positions on errors.

Before implementing Display or Error on a generic wrapper, inspect the wrapped
error's trait bounds. StockFlowError<K> requires K: Kind, not only Display.
