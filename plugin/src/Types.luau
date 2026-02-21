--!strict
-- StudioLink Type Definitions

export type PluginRequest = {
	id: string,
	tool: string,
	args: { [string]: any },
}

export type PluginResponse = {
	id: string,
	success: boolean,
	result: any,
	error: string?,
}

export type ToolHandler = (args: { [string]: any }) -> (boolean, any, string?)

export type SecurityIssue = {
	severity: string, -- "Critical" | "High" | "Medium" | "Low"
	category: string,
	description: string,
	location: string,
	suggestion: string,
}

export type DependencyNode = {
	path: string,
	requires: { string },
	requiredBy: { string },
}

export type MemoryIssue = {
	type: string,
	location: string,
	description: string,
	severity: string,
}

export type LintIssue = {
	rule: string,
	severity: string,
	line: number?,
	message: string,
	scriptPath: string,
}

export type SnapshotData = {
	name: string,
	timestamp: number,
	tree: { [string]: any },
}

return nil
