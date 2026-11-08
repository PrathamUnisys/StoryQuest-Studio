# StoryQuest Studio — Windows Task Scheduler Registration Helper
param(
    [Parameter(Mandatory=$true)][string]$ReminderId,
    [Parameter(Mandatory=$true)][string]$ScheduleTime,
    [string]$Frequency = "DAILY"
)

$AppExe = Join-Path $env:LOCALAPPDATA "StoryQuest Studio\storyquest-studio.exe"
$TaskName = "StoryQuestStudio\Reminder_$ReminderId"
$ActionCmd = "`"$AppExe`" --reminder-mode --reminder-id $ReminderId"

schtasks.exe /Create /F /SC $Frequency /TN $TaskName /TR $ActionCmd /ST $ScheduleTime
Write-Host "Registered Windows Scheduled Task: $TaskName at $ScheduleTime ($Frequency)" -ForegroundColor Green
