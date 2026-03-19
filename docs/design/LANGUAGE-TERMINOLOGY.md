# Ash Language Terminology Guide

This guide standardizes project wording for language and runtime concepts that are easy to
confuse across specs, plans, and implementation notes.

## Reserved Terms

- **policy**: Authorization and governance logic such as `Permit`, `Deny`,
  `RequireApproval`, or `Transform`. Do not use `policy` for scheduling, routing, or
  mailbox-selection behavior unless the paragraph explicitly redefines the term.
- **source scheduling modifier**: The language-level modifier that determines how
  `receive` chooses among eligible stream sources.
- **scheduler**: The runtime mechanism that implements a source scheduling modifier.

## Inputs

- **behaviour**: A continuous, time-varying input sampled with `observe`.
- **stream**: A discrete event input consumed with `receive`.
- **control mailbox**: The implicit workflow-owned mailbox used by `receive control`.
- **stream mailbox**: The queued input state for declared `receives` sources.

Workflow declarations should be explicit about input kinds:

```ash
workflow worker
    receives sensor:events, kafka:orders
    observes sensor:temperature, market:price
{
    ...
}
```

## Receive Terminology

- **receive arm order**: The order arms are written in the source.
- **source scheduling modifier**: The rule for selecting which eligible source is
  considered next.
- **priority**: The current default source scheduling modifier. It is biased by source
  order and may starve later arms or sources.
- **fair modifiers**: Terms such as `round_robin`, `random`, or `fair` refer to future
  source scheduling modifiers and should not be called policies.

## Links and Workflow Communication

- **InstanceAddr**: A communication endpoint for a workflow instance. It is an ordinary
  communicable value unless a later spec adds linearity constraints.
- **ControlLink**: Transferable control authority over a workflow instance.
- **control-link transfer**: Sending a `ControlLink` through a normal `send` operation.
  Transfer is **consume-on-success**: ownership is lost only after successful delivery.
  On failed send, the sender retains the link.

## Documentation Rule

When a paragraph is about workflow authorization, obligations, or capability permissions, use
`policy`. When it is about `receive` source selection or mailbox fairness, use
`source scheduling modifier` and `scheduler`.
