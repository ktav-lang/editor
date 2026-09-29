>>>>> lang=en
# TextMate Bundle Auto-Registration Problem in IntelliJ Plugins

> **Historical document.** Describes a rejected approach that tried to
> auto-register a bundled TextMate grammar at runtime. Since 0.2.0
> (2026-05-07) the plugin instead ships a native `KtavLexer` /
> `KtavSyntaxHighlighterFactory` (wired via `KtavParserDefinition`) for
> syntax highlighting — no TextMate bundle, no `KtavTextMateLoader`,
> no manual registration step. Kept for context on why that path was
> abandoned; statements below about a "current implementation" refer
> to the pre-0.2.0 attempt, not the shipped plugin.

## Summary

While developing the Ktav plugin for IntelliJ, we encountered a problem with automatically registering a bundled TextMate grammar. IntelliJ's TextMate API has significant limitations that make programmatic registration impossible or unreliable.

## What We Tried to Do

The goal was to enable syntax highlighting for `.ktav` files automatically when the plugin was installed, **without** requiring manual registration in the IDE settings.

## Solution Architecture

>>>>> lang=ru
# Проблема автоматической регистрации TextMate-бандла в плагинах IntelliJ

> **Исторический документ.** Описывает отвергнутый подход с попыткой
> автоматически регистрировать встроенную TextMate-грамматику во время
> выполнения. Начиная с 0.2.0 (2026-05-07) плагин вместо этого
> использует нативный `KtavLexer` / `KtavSyntaxHighlighterFactory`
> (подключены через `KtavParserDefinition`) для подсветки синтаксиса:
> нет TextMate-бандла, `KtavTextMateLoader` и шага ручной регистрации.
> Документ сохранён как объяснение отказа от прежнего пути; упоминания
> «текущей реализации» ниже относятся к попытке до 0.2.0, а не к
> выпущенному плагину.

## Краткое содержание

При разработке плагина Ktav для IntelliJ столкнулись с проблемой автоматической регистрации встроенной TextMate-грамматики. TextMate API IntelliJ имеет серьёзные ограничения, которые делают программную регистрацию невозможной или нестабильной.

## Что мы пытались сделать

Целью было обеспечить автоматическое включение подсветки синтаксиса для `.ktav` файлов при установке плагина **без** ручной регистрации в настройках IDE.

## Архитектура решения

>>>>> lang=zh
# IntelliJ 插件中 TextMate 包自动注册的问题

> **历史文档。** 本文记录一种已被否决的方案：运行时自动注册内置的
> TextMate 语法。从 0.2.0 (2026-05-07) 起，插件改用原生
> `KtavLexer` / `KtavSyntaxHighlighterFactory`，通过
> `KtavParserDefinition` 接入语法高亮；不再使用 TextMate 包、
> `KtavTextMateLoader`，也不需要手动注册。保留本文是为了解释
> 为何放弃原方案。下文所谓“当前实现”指 0.2.0 之前的尝试，
> 而非现已发布的插件。

## 概述

开发 Ktav IntelliJ 插件时，遇到了自动注册内置 TextMate 语法的问题。
IntelliJ 的 TextMate API 有重大限制，使程序化注册无法实现或不稳定。

## 我们试图实现什么

目标是在安装插件后自动为 `.ktav` 文件启用语法高亮，**无需**
用户通过 IDE 设置手动注册。

## 方案架构

