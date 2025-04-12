$action = New-ScheduledTaskAction -Execute "C:\Users\Pau\Desktop\UWU\act4.exe"

$trigger1 = New-ScheduledTaskTrigger -AtStartup
$trigger2 = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 5) -RepetitionDuration (New-TimeSpan -Days 2)

Register-ScheduledTask -TaskName "Actividad4uvu" `
    -Action $action `
    -Trigger @($trigger1, $trigger2) `
    -RunLevel Highest