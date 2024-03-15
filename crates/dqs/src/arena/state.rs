use derivative::Derivative;
use runtime::env::EnvironmentVariableStore;

use super::MainModule;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct ArenaRuntimeState {
  pub workspace_id: String,
  pub module: MainModule,
  pub env_variables: EnvironmentVariableStore,
}
