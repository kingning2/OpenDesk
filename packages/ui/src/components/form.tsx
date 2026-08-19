/**
 * Form — Zod schema + React Hook Form；FormInput 接到 shadcn Input。
 *
 * Feature 只提供 schema 与 submit；不要在页面里手写校验分支。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import { zodResolver } from "@hookform/resolvers/zod";
import * as React from "react";
import {
  Controller,
  FormProvider,
  useForm,
  useFormContext,
  type ControllerRenderProps,
  type DefaultValues,
  type FieldPath,
  type FieldValues,
  type UseFormReturn,
} from "react-hook-form";
import { z } from "zod";

import { cn } from "../lib/cn";
import { Input, type InputProps } from "./input";
import { Label } from "./label";
import { Textarea, type TextareaProps } from "./textarea";

/**
 * Form 属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type FormProps<TSchema extends z.ZodTypeAny> = {
  /** Zod schema，同时作为 RHF resolver。 */
  schema: TSchema;
  /** 提交回调；校验通过后才调用。 */
  onSubmit: (values: z.output<TSchema>) => void | Promise<void>;
  /** 默认值。 */
  defaultValues?: DefaultValues<z.input<TSchema>>;
  /** 表单内容，或接收 `useForm` 返回值的渲染函数。 */
  children: React.ReactNode | ((form: UseFormReturn<z.input<TSchema>, unknown, z.output<TSchema>>) => React.ReactNode);
  /** 表单 class。 */
  className?: string;
  /** 表单 id，供弹窗外按钮 `form` 属性提交。 */
  id?: string;
};

/**
 * 带 Zod 校验的表单根。
 *
 * 负责：
 * - `zodResolver` 接入 React Hook Form
 * - 向下提供 FormContext（FormInput 读取）
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link FormProps}
 * @returns 表单节点
 */
export function Form<TSchema extends z.ZodTypeAny>({
  schema,
  onSubmit,
  defaultValues,
  children,
  className,
  id,
}: FormProps<TSchema>) {
  const form = useForm<z.input<TSchema>, unknown, z.output<TSchema>>({
    resolver: zodResolver(schema),
    defaultValues,
  });

  return (
    <FormProvider {...form}>
      <form
        id={id}
        className={cn("grid gap-3", className)}
        onSubmit={form.handleSubmit(async (values) => {
          await onSubmit(values);
        })}
        noValidate
      >
        {typeof children === "function" ? children(form) : children}
      </form>
    </FormProvider>
  );
}

/**
 * FormInput 属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type FormInputProps<TFieldValues extends FieldValues> = Omit<InputProps, "name" | "defaultValue"> & {
  /** 字段名，对应 schema key。 */
  name: FieldPath<TFieldValues>;
  /** 标签。 */
  label?: string;
  /** 辅助说明；有校验错误时被错误文案替换。 */
  description?: string;
};

function FieldMessage({ error, description }: { error?: string; description?: string }) {
  if (error) {
    return (
      <p className="text-(length:--text-xs) text-destructive" role="alert">
        {error}
      </p>
    );
  }
  if (description) {
    return <p className="text-(length:--text-xs) text-muted-foreground">{description}</p>;
  }
  return null;
}

function toInputValue(value: unknown): string | number | readonly string[] | undefined {
  if (value === undefined || value === null) {
    return "";
  }
  if (typeof value === "string" || typeof value === "number") {
    return value;
  }
  if (Array.isArray(value) && value.every((item) => typeof item === "string")) {
    return value;
  }
  return String(value);
}

/**
 * 绑定 RHF 的单行输入：FormInput → Input → React Hook Form → Zod。
 *
 * 必须放在 {@link Form} 内。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link FormInputProps}
 * @returns 带标签与错误态的输入
 */
export function FormInput<TFieldValues extends FieldValues = FieldValues>({
  name,
  label,
  description,
  id,
  className,
  ...inputProps
}: FormInputProps<TFieldValues>) {
  const { control } = useFormContext<TFieldValues>();
  const inputId = id ?? String(name);

  return (
    <Controller
      name={name}
      control={control}
      render={({ field, fieldState }) => (
        <div className="grid gap-1.5">
          {label ? <Label htmlFor={inputId}>{label}</Label> : null}
          <Input
            {...inputProps}
            {...bindField(field)}
            id={inputId}
            className={className}
            value={toInputValue(field.value)}
            aria-invalid={fieldState.invalid || undefined}
          />
          <FieldMessage error={fieldState.error?.message} description={description} />
        </div>
      )}
    />
  );
}

/**
 * FormTextarea 属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type FormTextareaProps<TFieldValues extends FieldValues> = Omit<TextareaProps, "name" | "defaultValue"> & {
  /** 字段名，对应 schema key。 */
  name: FieldPath<TFieldValues>;
  /** 标签。 */
  label?: string;
  /** 辅助说明。 */
  description?: string;
};

/**
 * 绑定 RHF 的多行输入（与 FormInput 同一套校验链路）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link FormTextareaProps}
 * @returns 带标签与错误态的文本域
 */
export function FormTextarea<TFieldValues extends FieldValues = FieldValues>({
  name,
  label,
  description,
  id,
  className,
  ...textareaProps
}: FormTextareaProps<TFieldValues>) {
  const { control } = useFormContext<TFieldValues>();
  const inputId = id ?? String(name);

  return (
    <Controller
      name={name}
      control={control}
      render={({ field, fieldState }) => (
        <div className="grid gap-1.5">
          {label ? <Label htmlFor={inputId}>{label}</Label> : null}
          <Textarea
            {...textareaProps}
            {...bindField(field)}
            id={inputId}
            className={className}
            value={toInputValue(field.value)}
            aria-invalid={fieldState.invalid || undefined}
          />
          <FieldMessage error={fieldState.error?.message} description={description} />
        </div>
      )}
    />
  );
}

function bindField<TFieldValues extends FieldValues>(
  field: ControllerRenderProps<TFieldValues, FieldPath<TFieldValues>>,
): Pick<ControllerRenderProps<TFieldValues, FieldPath<TFieldValues>>, "name" | "onBlur" | "onChange" | "ref"> {
  return {
    name: field.name,
    onBlur: field.onBlur,
    onChange: field.onChange,
    ref: field.ref,
  };
}
