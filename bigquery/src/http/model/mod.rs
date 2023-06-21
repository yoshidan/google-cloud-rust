use std::collections::HashMap;

use time::OffsetDateTime;

use crate::http::table::TableReference;
use crate::http::types::{EncryptionConfiguration, StandardSqlField};

pub mod delete;
pub mod get;
pub mod list;
pub mod patch;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Output only. A hash of this resource.
    pub etag: String,
    /// Required. Unique identifier for this model.
    pub model_reference: ModelReference,
    /// Output only. The time when this model was created, in millisecs since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Output only. The time when this model was last modified, in millisecs since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub last_modified_time: u64,
    /// Optional. A user-friendly description of this model.
    pub description: Option<String>,
    /// Optional. A descriptive name for this model.
    pub friendly_name: Option<String>,
    /// The labels associated with this model.
    /// You can use these to organize and group your models. Label keys and values can be no longer than 63 characters,
    /// can only contain lowercase letters, numeric characters, underscores and dashes.
    /// International characters are allowed. Label values are optional.
    /// Label keys must start with a letter and each label in the list must have a different key.
    /// An object containing a list of "key": value pairs. Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<HashMap<String, String>>,
    /// Optional. The time when this model expires, in milliseconds since the epoch.
    /// If not present, the model will persist indefinitely. Expired models will be deleted and their storage reclaimed. The defaultTableExpirationMs property of the encapsulating dataset can be used to set a default expirationTime on newly created models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub expiration_time: Option<i64>,
    /// Output only. The geographic location where the model resides. This value is inherited from the dataset.
    pub location: Option<String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    /// This shows the encryption configuration of the model data while stored in BigQuery storage.
    /// This field can be used with models.patch to update encryption key for an already encrypted model.
    pub encryption_configuration: Option<EncryptionConfiguration>,
    /// Output only. Type of the model resource.
    pub model_type: Option<ModelType>,
    /// Information for all training runs in increasing order of startTime.
    pub training_runs: Option<Vec<TrainingRun>>,
    /// Output only. Input feature columns that were used to train this model.
    pub feature_columns: Option<Vec<StandardSqlField>>,
    /// Output only. Label columns that were used to train this model.
    /// The output of the model will have a "predicted_" prefix to these columns.
    pub label_columns: Option<Vec<StandardSqlField>>,
    /// Output only. All hyperparameter search spaces in this model.
    pub hparam_search_spaces: Option<HparamSearchSpaces>,
    /// Output only. The default trialId to use in TVFs when the trialId is not passed in. For single-objective hyperparameter tuning models, this is the best trial ID. For multi-objective hyperparameter tuning models, this is the smallest trial ID among all Pareto optimal trials.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub default_trial_id: Option<i64>,
    /// Output only. Trials of a hyperparameter tuning model sorted by trialId.
    pub hparam_trials: Option<Vec<HparamTuningTrial>>,
    /// Output only. For single-objective hyperparameter tuning models, it only contains the best trial.
    /// For multi-objective hyperparameter tuning models, it contains all Pareto optimal trials sorted by trialId.
    #[serde(default, deserialize_with = "crate::http::from_str_vec_option")]
    pub optimal_trial_ids: Option<Vec<i64>>,
    /// Output only. Remote model info
    pub remote_model_info: Option<RemoteModelInfo>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrainingRun {
    /// Output only. Options that were used for this training run, includes user specified and default options that were used.
    pub training_options: Option<TrainingOptions>,
    /// Output only. The start time of this training run.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub start_time: Option<OffsetDateTime>,
    /// Output only. Output of each iteration run, results.size() <= maxIterations.
    pub results: Option<Vec<IterationResult>>,
    /// Output only. The evaluation metrics over training/eval data that were computed at the end of training.
    pub evaluation_metrics: Option<EvaluationMetrics>,
    /// Output only. Data split result of the training run. Only set when the input data is actually split.
    pub data_split_result: Option<DataSplitResult>,
    /// Output only. Global explanation contains the explanation of top features on the model level. Applies to both regression and classification models.
    pub model_level_global_explanation: Option<GlobalExplanation>,
    /// Output only. Global explanation contains the explanation of top features on the class level. Applies to classification models only.
    pub class_level_global_explanations: Option<Vec<GlobalExplanation>>,
    /// The model id in the Vertex AI Model Registry for this training run.
    pub vertex_ai_model_id: Option<String>,
    /// Output only. The model version in the Vertex AI Model Registry for this training run.
    pub vertex_ai_model_version: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataSplitResult {
    /// Table reference of the training data after split.
    pub training_table: Option<TableReference>,
    /// Table reference of the evaluation data after split.
    pub evaluation_table: Option<TableReference>,
    /// Table reference of the test data after split.
    pub test_table: Option<TableReference>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GlobalExplanation {
    /// A list of the top global explanations. Sorted by absolute value of attribution in descending order.
    pub explanations: Option<Vec<Explanation>>,
    /// Class label for this set of global explanations.
    /// Will be empty/null for binary logistic and linear regression models. Sorted alphabetically in descending order.
    pub class_label: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    /// The full feature name.
    /// For non-numerical features, will be formatted like <column_name>.<encoded_feature_name>.
    /// Overall size of feature name will always be truncated to first 120 characters..
    pub feature_name: String,
    /// Attribution of feature.
    pub attribution: f64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModelInfo {
    /// Output only. Fully qualified name of the user-provided connection object of the remote model.
    /// Format: "projects/{projectId}/locations/{locationId}/connections/{connectionId}"
    pub connection: String,
    /// Output only. Max number of rows in each batch sent to the remote service. If unset, the number of rows in each batch is set dynamically.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub max_batching_rows: Option<i64>,
    /// Output only. The endpoint for remote model.
    pub endpoint: String,
    /// Output only. The remote service type for remote model.
    pub remote_service_type: RemoteServiceType,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HparamSearchSpaces {
    /// Learning rate of training jobs.
    pub learn_rate: Option<DoubleHparamSearchSpace>,
    /// L1 regularization coefficient.
    pub l1_reg: Option<DoubleHparamSearchSpace>,
    /// L2 regularization coefficient.
    pub l2_reg: Option<DoubleHparamSearchSpace>,
    /// Number of clusters for k-means.
    pub num_clusters: Option<IntHparamSearchSpace>,
    /// Number of latent factors to train on.
    pub num_factors: Option<IntHparamSearchSpace>,
    /// Hidden units for neural network models.
    pub hidden_units: Option<IntArrayHparamSearchSpace>,
    /// Mini batch sample size.
    pub batch_size: Option<IntHparamSearchSpace>,
    /// Dropout probability for dnn model training and boosted tree models using dart booster.
    pub dropout: Option<DoubleHparamSearchSpace>,
    /// Maximum depth of a tree for boosted tree models.
    pub max_tree_depth: Option<IntHparamSearchSpace>,
    /// Subsample the training data to grow tree to prevent overfitting for boosted tree models.
    pub subsample: Option<DoubleHparamSearchSpace>,
    /// Minimum split loss for boosted tree models.
    pub min_split_loss: Option<DoubleHparamSearchSpace>,
    /// Hyperparameter for matrix factoration when implicit feedback type is specified.
    pub wals_alpha: Option<DoubleHparamSearchSpace>,
    /// Booster type for boosted tree models.
    pub booster_type: Option<StringHparamSearchSpace>,
    /// Number of parallel trees for boosted tree models.
    pub num_parallel_tree: Option<IntHparamSearchSpace>,
    /// Dart normalization type for boosted tree models.
    pub dart_normalize_type: Option<StringHparamSearchSpace>,
    /// Tree construction algorithm for boosted tree models.
    pub tree_method: Option<StringHparamSearchSpace>,
    /// Minimum sum of instance weight needed in a child for boosted tree models.
    pub min_tree_child_weight: Option<IntHparamSearchSpace>,
    /// Subsample ratio of columns when constructing each tree for boosted tree models.
    pub colsample_bytree: Option<DoubleHparamSearchSpace>,
    /// Subsample ratio of columns for each level for boosted tree models.
    pub colsample_bylevel: Option<DoubleHparamSearchSpace>,
    /// Subsample ratio of columns for each node(split) for boosted tree models.
    pub colsample_bynode: Option<DoubleHparamSearchSpace>,
    /// Activation functions of neural network models.
    pub activation_fn: Option<StringHparamSearchSpace>,
    /// Optimizer of TF models.
    pub optimizer: Option<StringHparamSearchSpace>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DoubleHparamSearchSpace {
    Range(DoubleRange),
    Candidates(DoubleCandidates),
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoubleRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoubleCandidates {
    pub candidates: Vec<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum IntHparamSearchSpace {
    Range(IntRange),
    Candidates(IntCandidates),
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntRange {
    #[serde(deserialize_with = "crate::http::from_str")]
    pub min: i64,
    #[serde(deserialize_with = "crate::http::from_str")]
    pub max: i64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntCandidates {
    #[serde(deserialize_with = "crate::http::from_str_vec")]
    pub candidates: Vec<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntArrayHparamSearchSpace {
    pub candidates: Vec<IntArray>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntArray {
    #[serde(deserialize_with = "crate::http::from_str")]
    pub elements: i64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StringHparamSearchSpace {
    pub candidates: Vec<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RemoteServiceType {
    #[default]
    RemoteServiceTypeUnspecified,
    CloudAiTranslateV3,
    CloudAiVisionV1,
    CloudAiNaturalLanguageV1,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelReference {
    /// Required. The ID of the project containing this table.
    pub project_id: String,
    /// Required. The ID of the dataset containing this table.
    pub dataset_id: String,
    /// Required. The ID of the model.
    /// The ID must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_). The maximum length is 1,024 characters.    pub model_id: String,
    pub model_id: String,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IterationResult {
    /// Index of the iteration, 0 based.
    pub index: i32,
    /// Time taken to run the iteration in milliseconds.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub duration_ms: Option<i64>,
    /// Loss computed on the training data at the end of iteration.
    pub training_loss: Option<f64>,
    /// Loss computed on the eval data at the end of iteration.
    pub eval_loss: Option<f64>,
    /// Learn rate used for this iteration.
    pub learn_rate: Option<f64>,
    /// Information about top clusters for clustering models.
    pub cluster_infos: Option<Vec<ClusterInfo>>,
    pub arima_result: Option<ArimaResult>,
    /// The information of the principal components.
    pub principal_component_infos: Option<Vec<PrincipalComponentInfo>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    /// Centroid id.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub centroid_id: Option<i64>,
    /// Cluster radius, the average distance from centroid to each point assigned to the cluster.
    pub cluster_radius: Option<f64>,
    /// Cluster size, the total number of points assigned to the cluster.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub cluster_size: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaResult {
    /// This message is repeated because there are multiple arima models fitted in auto-arima. For non-auto-arima model, its size is one.
    pub arima_model_info: Option<Vec<ArimaModelInfo>>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    pub seasonal_periods: Option<Vec<SeasonalPeriodType>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaModelInfo {
    /// Non-seasonal order.
    pub non_seasonal_order: Option<ArimaOrder>,
    /// Arima coefficients.
    pub arima_coefficients: Option<ArimaCoefficients>,
    /// Arima fitting metrics.
    pub arima_fitting_metrics: Option<ArimaFittingMetrics>,
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    pub has_drift: Option<bool>,
    /// The timeSeriesId value for this time series.
    /// It will be one of the unique values from the timeSeriesIdColumn specified during ARIMA model training.
    /// Only present when timeSeriesIdColumn training option was used.
    pub time_series_id: Option<String>,
    /// The tuple of timeSeriesIds identifying this time series.
    /// It will be one of the unique tuples of values present in the timeSeriesIdColumns specified
    /// during ARIMA model training.
    /// Only present when timeSeriesIdColumns training option was used and
    /// the order of values here are same as the order of timeSeriesIdColumns.
    pub time_series_ids: Option<Vec<String>>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    pub seasonal_periods: Option<Vec<SeasonalPeriodType>>,
    /// If true, holiday_effect is a part of time series decomposition result.
    pub has_holiday_effect: Option<bool>,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    pub has_spikes_and_dips: Option<bool>,
    /// If true, step_changes is a part of time series decomposition result.
    pub has_step_changes: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaCoefficients {
    /// Auto-regressive coefficients, an array of double.
    pub auto_regressive_coefficients: Option<Vec<f64>>,
    /// Moving-average coefficients, an array of double.
    pub moving_average_coefficients: Option<Vec<f64>>,
    /// Intercept coefficient, just a double not an array
    pub intercept_coefficient: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaFittingMetrics {
    /// Log-likelihood.
    pub log_likelihood: Option<f64>,
    /// AIC.
    pub aic: Option<f64>,
    /// Variance.
    pub variance: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeasonalPeriodType {
    #[default]
    SeasonalPeriodTypeUnspecified,
    NoSeasonality,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalComponentInfo {
    /// Id of the principal component.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub principal_component_id: i64,
    /// Explained variance by this principal component, which is simply the eigenvalue.
    pub explained_variance: Option<f64>,
    /// Explained_variance over the total explained variance.
    pub explained_variance_ratio: Option<f64>,
    /// The explainedVariance is pre-ordered in the descending order to compute
    /// the cumulative explained variance ratio.
    pub cumulative_explained_variance_ratio: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelType {
    #[default]
    ModelTypeUnspecified,
    /// Linear regression model.
    LinearRegression,
    ///Logistic regression based classification model.
    LogisticRegression,
    /// K-means clustering model.
    Kmeans,
    /// Matrix factorization model.
    MatrixFactorization,
    /// DNN classifier model.
    DnnClassifier,
    ///An imported TensorFlow model.
    Tensorflow,
    /// DNN regressor model.
    DnnRegression,
    /// Boosted tree regressor model.
    BoostedTreeRegressor,
    /// Boosted tree classifier model.
    BoostedTreeClassifier,
    ///Arima Model
    Arima,
    /// AutoML Tables regression model.
    AutomlRegressor,
    /// AutoML Tables classification model.
    AutomlClassifier,
    /// Prinpical Component Analysis model.
    Pca,
    /// Autoencoder model.
    Autoencoder,
    /// New name for the ARIMA model.
    ArimaPlus,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HparamTuningTrial {
    /// 1-based index of the trial.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub trial_id: Option<i64>,
    /// Starting time of the trial.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub start_time_ms: Option<i64>,
    /// Ending time of the trial.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub end_time_ms: Option<i64>,
    /// The hyperprameters selected for this trial.
    pub hparams: Option<TrainingOptions>,
    /// Evaluation metrics of this trial calculated on the test data. Empty in Job API.
    pub evaluation_metrics: Option<EvaluationMetrics>,
    /// The status of the trial.
    pub status: Option<TrialStatus>,
    /// Error message for FAILED and INFEASIBLE trial.
    pub error_message: Option<String>,
    /// Loss computed on the training data at the end of trial.
    pub training_loss: Option<f64>,
    /// Loss computed on the eval data at the end of trial.
    pub eval_loss: Option<f64>,
    /// Hyperparameter tuning evaluation metrics of this trial calculated on the eval data
    /// . Unlike evaluationMetrics, only the fields corresponding to the hparamTuningObjectives are set.
    pub hparam_tuning_evaluation_metrics: Option<EvaluationMetrics>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrialStatus {
    #[default]
    TrialStatusUnspecified,
    NotStarted,
    Running,
    Succeeded,
    Failed,
    Infeasible,
    StoppedEarly,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LossType {
    #[default]
    LossTypeUnspecified,
    MeanSquaredLoss,
    MeanLogLoss,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataSplitMethod {
    #[default]
    DataSplitMethodUnspecified,
    Random,
    Custom,
    Sequential,
    NoSplit,
    AutoSplit,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LearnRateStrategy {
    #[default]
    LearnRateStrategyUnspecified,
    LineSearch,
    Constant,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DistanceType {
    #[default]
    DistanceTypeUnspecified,
    Euclidean,
    Cosine,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptimizationStrategy {
    #[default]
    OptimizationStrategyUnspecified,
    BatchGradientDescent,
    NormalEquation,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BoosterType {
    #[default]
    BoosterTypeUnspecified,
    Gbtree,
    Dart,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DartNormalizeType {
    #[default]
    DataNormalizeTypeUnspecified,
    Tree,
    Forest,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TestMethod {
    #[default]
    TreeMethodUnspecified,
    Auto,
    Exact,
    Approx,
    Hist,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeedbackType {
    #[default]
    FeedbackTypeUnspecified,
    Implicit,
    Explicit,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KmeansInitializationMethod {
    #[default]
    KmeansInitializationMethodUnspecified,
    Random,
    Custom,
    KmeansPlusPlus,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaOrder {
    /// Order of the autoregressive part.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub p: Option<i64>,
    /// Order of the differencing part.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub d: Option<i64>,
    /// Order of the moving-average part.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub q: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataFrequency {
    #[default]
    DataFrequencyUnspecified,
    AutoFrequency,
    Yearly,
    Quarterly,
    Monthly,
    Weekly,
    Daily,
    Hourly,
    PerMinute,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HolidayRegion {
    #[default]
    HolidayRegionUnspecified,
    Global,
    Na,
    Japac,
    Emea,
    Ae,
    Ar,
    At,
    Au,
    Be,
    Br,
    Ca,
    Ch,
    Cl,
    Cn,
    Co,
    Cs,
    Cz,
    De,
    Dk,
    Dz,
    Ec,
    Ee,
    Eg,
    Es,
    Fi,
    Fr,
    Gb,
    Gr,
    Hk,
    Hu,
    Id,
    Ie,
    Il,
    In,
    Ir,
    It,
    Jp,
    Kr,
    Lv,
    Ma,
    Mx,
    My,
    Mg,
    Nl,
    No,
    Nz,
    Pe,
    Ph,
    Pk,
    Pl,
    Pt,
    Ro,
    Rs,
    Ru,
    Sa,
    Se,
    Sg,
    Si,
    Sk,
    Th,
    Tr,
    Tw,
    Ua,
    Us,
    Ve,
    Vn,
    Za,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HparamTuningObjective {
    #[default]
    HparamTuningObjectiveUnspecified,
    MeanAbsoluteError,
    MeanSquaredError,
    MeanSquaredLogError,
    MedianAbsoluteError,
    RSquared,
    ExplainedVariance,
    Precision,
    Recall,
    Accuracy,
    F1Score,
    LogLoss,
    RocAuc,
    DaviesBouldinIndex,
    MeanAveragePrecision,
    NormalizedDiscountedCumulativeGain,
    AverageRank,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TreeMethod {
    #[default]
    TreeMethodUnspecified,
    Auto,
    Exact,
    Approx,
    Hist,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrainingOptions {
    /// The maximum number of iterations in training. Used only for iterative training algorithms.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub max_iterations: Option<i64>,
    /// Type of loss function used during training run.
    pub loss_type: Option<LossType>,
    /// Learning rate in training. Used only for iterative training algorithms.
    pub learn_rate: Option<f64>,
    /// L1 regularization coefficient.
    pub l1_regularization: Option<f64>,
    /// L2 regularization coefficient.
    pub l2_regularization: Option<f64>,
    /// When earlyStop is true, stops training when accuracy improvement is less than 'minRelativeProgress'. Used only for iterative training algorithms.
    pub min_relative_progress: Option<f64>,
    /// Whether to train a model from the last checkpoint.
    pub warm_start: Option<bool>,
    /// Whether to stop early when the loss doesn't improve significantly any more (compared to minRelativeProgress). Used only for iterative training algorithms.
    pub early_stop: Option<bool>,
    /// Name of input label columns in training data.
    pub input_label_columns: Option<Vec<String>>,
    /// The data split type for training and evaluation, e.g. RANDOM.
    pub data_split_method: Option<DataSplitMethod>,
    /// The fraction of evaluation data over the whole input data.
    /// The rest of data will be used as training data. The format should be double. Accurate to two decimal places. Default value is 0.2.
    pub data_split_eval_fraction: Option<f64>,
    /// The column to split data with. This column won't be used as a feature.
    /// 1. When dataSplitMethod is CUSTOM, the corresponding column should be boolean.
    /// The rows with true value tag are eval data, and the false are training data.
    /// 2. When dataSplitMethod is SEQ, the first DATA_SPLIT_EVAL_FRACTION rows (from smallest to largest) in the corresponding column are used as training data,
    /// and the rest are eval data.
    /// It respects the order in Orderable data types: https://cloud.google.com/bigquery/docs/reference/standard-sql/data-types#data-type-properties
    pub data_split_column: Option<String>,
    /// The strategy to determine learn rate for the current iteration.
    pub learn_rate_strategy: Option<LearnRateStrategy>,
    /// Specifies the initial learning rate for the line search learn rate strategy.
    pub initial_learn_rate: Option<f64>,
    /// Weights associated with each label class, for rebalancing the training data.
    /// Only applicable for classification models.
    /// An object containing a list of "key": value pairs. Example:
    /// { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub label_class_weights: Option<HashMap<String, f64>>,
    /// User column specified for matrix factorization models.
    pub user_column: Option<String>,
    /// Item column specified for matrix factorization models.
    pub item_column: Option<String>,
    /// Distance type for clustering models.
    pub distance_type: Option<DistanceType>,
    /// Number of clusters for clustering models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_clusters: Option<i64>,
    /// Google Cloud Storage URI from which the model was imported.
    /// Only applicable for imported models.
    pub model_uri: Option<String>,
    /// Optimization strategy for training linear regression models.
    pub optimization_strategy: Option<OptimizationStrategy>,
    /// Hidden units for dnn models.
    #[serde(default, deserialize_with = "crate::http::from_str_vec_option")]
    pub hidden_units: Option<Vec<i64>>,
    /// Batch size for dnn models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub batch_size: Option<i64>,
    /// Dropout probability for dnn models.
    pub dropout: Option<f64>,
    /// Maximum depth of a tree for boosted tree models.
    pub max_tree_depth: Option<i64>,
    /// Subsample fraction of the training data to grow tree to prevent overfitting for boosted tree models.
    pub subsample: Option<f64>,
    /// Minimum split loss for boosted tree models.
    pub min_split_loss: Option<f64>,
    /// Booster type for boosted tree models.
    pub booster_type: Option<BoosterType>,
    /// Number of parallel trees constructed during each iteration for boosted tree models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_parallel_tree: Option<i64>,
    /// Type of normalization algorithm for boosted tree models using dart booster.
    pub dart_normalize_type: Option<DartNormalizeType>,
    /// Tree construction algorithm for boosted tree models.
    pub tree_method: Option<TreeMethod>,
    /// Minimum sum of instance weight needed in a child for boosted tree models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub min_tree_child_weight: Option<i64>,
    /// Subsample ratio of columns when constructing each tree for boosted tree models.
    pub colsample_bytree: Option<f64>,
    /// Subsample ratio of columns for each level for boosted tree models.
    pub colsample_bylevel: Option<f64>,
    /// Subsample ratio of columns for each node(split) for boosted tree models.
    pub colsample_bynode: Option<f64>,
    /// Num factors specified for matrix factorization models.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_factors: Option<i64>,
    /// Feedback type that specifies which algorithm to run for matrix factorization.
    pub feedback_type: Option<FeedbackType>,
    /// Hyperparameter for matrix factoration when implicit feedback type is specified.
    pub wals_alpha: Option<f64>,
    /// The method used to initialize the centroids for kmeans algorithm.
    pub kmeans_initialization_method: Option<KmeansInitializationMethod>,
    /// The column used to provide the initial centroids for kmeans algorithm when kmeansInitializationMethod is CUSTOM.
    pub kmeans_initialization_column: Option<String>,
    /// Column to be designated as time series timestamp for ARIMA model.
    pub time_series_timestamp_column: Option<String>,
    /// Column to be designated as time series data for ARIMA model.
    pub time_series_data_column: Option<String>,
    /// Whether to enable auto ARIMA or not.
    pub auto_arima: Option<bool>,
    /// A specification of the non-seasonal part of the ARIMA model: the three components (p, d, q) are the AR order, the degree of differencing, and the MA order.
    pub non_seasonal_order: Option<ArimaOrder>,
    /// The data frequency of a time series.
    pub data_frequency: Option<DataFrequency>,
    /// Whether or not p-value test should be computed for this model. Only available for linear and logistic regression models.
    pub calculate_p_values: Option<bool>,
    /// Include drift when fitting an ARIMA model.
    pub include_drift: Option<bool>,
    /// The geographical region based on which the holidays are considered in time series modeling. If a valid value is specified, then holiday effects modeling is enabled.
    pub holiday_region: Option<HolidayRegion>,
    /// The time series id column that was used during ARIMA model training.
    pub time_series_id_column: Option<String>,
    /// The time series id columns that were used during ARIMA model training.
    pub time_series_id_columns: Option<Vec<String>>,
    /// The number of periods ahead that need to be forecasted.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub horizon: Option<i64>,
    /// Whether to preserve the input structs in output feature names.
    /// Suppose there is a struct A with field b. When false (default), the output feature name is A_b. When true, the output feature name is A.b.
    pub preserve_input_structs: Option<bool>,
    /// The max value of the sum of non-seasonal p and q.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub auto_arima_max_order: Option<i64>,
    /// The min value of the sum of non-seasonal p and q.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub auto_arima_min_order: Option<i64>,
    /// Number of trials to run this hyperparameter tuning job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_trials: Option<i64>,
    /// Maximum number of trials to run in parallel.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub max_parallel_trials: Option<i64>,
    /// The target evaluation metrics to optimize the hyperparameters for.
    pub hparam_tuning_objectives: Option<Vec<HparamTuningObjective>>,
    /// If true, perform decompose time series and save the results.
    pub decompose_time_series: Option<bool>,
    /// If true, clean spikes and dips in the input time series.
    pub clean_spikes_and_dips: Option<bool>,
    /// If true, detect step changes and make data adjustment in the input time series.
    pub adjust_step_changes: Option<bool>,
    /// If true, enable global explanation during training.
    pub enable_global_explain: Option<bool>,
    /// Number of paths for the sampled Shapley explain method.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub sampled_shapley_num_paths: Option<i64>,
    /// Number of integral steps for the integrated gradients explain method.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub integrated_gradients_num_steps: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum EvaluationMetrics {
    /// Populated for regression models and explicit feedback type matrix factorization models.
    RegressionMetrics(RegressionMetrics),
    /// Populated for binary classification/classifier models.
    BinaryClassificationMetrics(BinaryClassificationMetrics),
    /// Populated for multi-class classification/classifier models.
    MultiClassClassificationMetrics(MultiClassClassificationMetrics),
    /// Populated for clustering models.
    ClusteringMetrics(ClusteringMetrics),
    /// Populated for implicit feedback type matrix factorization models.
    RankingMetrics(RankingMetrics),
    /// Populated for ARIMA models.
    ArimaForecastingMetrics(ArimaForecastingMetrics),
    /// Evaluation metrics when the model is a dimensionality reduction model, which currently includes PCA.
    DimensionalityReductionMetrics(DimensionalityReductionMetrics),
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self::RegressionMetrics(RegressionMetrics::default())
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegressionMetrics {
    /// Mean absolute error.
    pub mean_absolute_error: Option<f64>,
    /// Mean squared error.
    pub mean_squared_error: Option<f64>,
    /// Mean squared log error.
    pub mean_squared_log_error: Option<f64>,
    /// Median absolute error.
    pub median_absolute_error: Option<f64>,
    /// R^2 score. This corresponds to r2_score in ML.EVALUATE.
    pub r_squared: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationMetrics {
    /// Aggregate classification metrics.
    pub aggregate_classification_metrics: Option<AggregateClassificationMetrics>,
    /// Binary confusion matrix at multiple thresholds.
    pub binary_confusion_matrix_list: Option<Vec<BinaryConfusionMatrix>>,
    /// Label representing the positive class.
    pub positive_label: Option<String>,
    /// Label representing the negative class.
    pub negative_label: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AggregateClassificationMetrics {
    /// Precision is the fraction of actual positive predictions that had positive actual labels.
    /// For multiclass this is a macro-averaged metric treating each class as a binary classifier.
    pub precision: Option<f64>,
    /// Recall is the fraction of actual positive labels that were given a positive prediction.
    /// For multiclass this is a macro-averaged metric.
    pub recall: Option<f64>,
    /// Accuracy is the fraction of predictions given the correct label. For multiclass this is a micro-averaged metric.
    pub accuracy: Option<f64>,
    /// Threshold at which the metrics are computed.
    /// For binary classification models this is the positive class threshold.
    /// For multi-class classfication models this is the confidence threshold.
    pub threshold: Option<f64>,
    /// The F1 score is an average of recall and precision. For multiclass this is a macro-averaged metric.
    pub f1_score: Option<f64>,
    /// Logarithmic Loss. For multiclass this is a macro-averaged metric.
    pub log_loss: Option<f64>,
    /// Area Under a ROC Curve. For multiclass this is a macro-averaged metric.
    pub roc_auc: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BinaryConfusionMatrix {
    /// Threshold value used when computing each of the following metric.
    pub positive_class_threshold: Option<f64>,
    /// Number of true samples predicted as true.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub true_positives: Option<i64>,
    /// Number of false samples predicted as true.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub false_positives: Option<i64>,
    /// Number of true samples predicted as false.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub true_negatives: Option<i64>,
    /// Number of false samples predicted as false.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub false_negatives: Option<i64>,
    /// The fraction of actual positive predictions that had positive actual labels.
    pub precision: Option<f64>,
    /// The fraction of actual positive labels that were given a positive prediction.
    pub recall: Option<f64>,
    /// The equally weighted average of recall and precision.
    pub f1_score: Option<f64>,
    /// The fraction of predictions given the correct label.
    pub accuracy: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MultiClassClassificationMetrics {
    /// Aggregate classification metrics.
    pub aggregate_classification_metrics: Option<AggregateClassificationMetrics>,
    /// Confusion matrix at different thresholds.
    pub confusion_matrix_list: Option<Vec<ConfusionMatrix>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfusionMatrix {
    /// Confidence threshold used when computing the entries of the confusion matrix.
    pub confidence_threshold: Option<f64>,
    /// One row per actual label.
    pub rows: Option<Vec<Row>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// The original label of this row.
    pub actual_label: Option<String>,
    /// Info describing predicted label distribution.
    pub entries: Option<Vec<Entry>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// The predicted label. For confidenceThreshold > 0,
    /// we will also add an entry indicating the number of items under the confidence threshold.
    pub predicted_label: Option<String>,
    /// Number of items being predicted as this label.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub item_count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusteringMetrics {
    /// Davies-Bouldin index.
    pub davies_bouldin_index: Option<f64>,
    /// Mean of squared distances between each sample to its cluster centroid.
    pub mean_squared_distance: Option<f64>,
    /// Information for all clusters.
    pub clusters: Option<Vec<Cluster>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    /// Centroid id.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub centroid_id: Option<i64>,
    /// Values of highly variant features for this cluster.
    pub feature_values: Option<Vec<FeatureValue>>,
    /// Count of training data rows that were assigned to this cluster.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureValue {
    /// The feature column name.
    pub feature_column: Option<String>,
    #[serde(flatten)]
    pub value: FeatureValueType,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FeatureValueType {
    /// The numerical feature value. This is the centroid value for this feature.
    NumericalValue(f64),
    /// The categorical feature value.
    CategoricalValue(CategoricalValue),
}

impl Default for FeatureValueType {
    fn default() -> Self {
        Self::NumericalValue(0.0)
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalValue {
    /// Counts of all categories for the categorical feature.
    /// If there are more than ten categories, we return top ten (by count) and return one more
    /// CategoryCount with category "_OTHER_" and count as aggregate counts of remaining categories.
    pub category_counts: Option<Vec<CategoryCount>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    /// The name of category.
    pub category: Option<String>,
    /// The count of training samples matching the category within the cluster.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RankingMetrics {
    /// Calculates a precision per user for all the items by ranking them and then averages all the precisions across all the users.
    pub mean_average_precision: Option<f64>,
    /// Similar to the mean squared error computed in regression and explicit recommendation models except instead of computing the rating directly, the output from evaluate is computed against a preference which is 1 or 0 depending on if the rating exists or not.
    pub mean_squared_error: Option<f64>,
    /// A metric to determine the goodness of a ranking calculated from the predicted confidence by comparing it to an ideal rank measured by the original ratings.
    pub normalized_discounted_cumulative_gain: Option<f64>,
    /// Determines the goodness of a ranking by computing the percentile rank from the predicted confidence and dividing it by the original rank.
    pub average_rank: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaForecastingMetrics {
    /// Repeated as there can be many metric sets (one for each model) in auto-arima and the large-scale case.
    pub arima_single_model_forecasting_metrics: Option<Vec<ArimaSingleModelForecastingMetrics>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaSingleModelForecastingMetrics {
    /// Non-seasonal order.
    pub non_seasonal_order: Option<ArimaOrder>,
    /// Arima fitting metrics.
    pub arima_fitting_metrics: Option<ArimaFittingMetrics>,
    /// Is arima model fitted with drift or not. It is always false when d is not 1.
    pub has_drift: Option<bool>,
    /// The timeSeriesId value for this time series. It will be one of the unique values from the timeSeriesIdColumn specified during ARIMA model training. Only present when timeSeriesIdColumn training option was used.
    pub time_series_id: Option<String>,
    /// The tuple of timeSeriesIds identifying this time series.
    /// It will be one of the unique tuples of values present in the timeSeriesIdColumns specified during ARIMA model training. Only present when timeSeriesIdColumns training option was used and the order of values here are same as the order of timeSeriesIdColumns.
    pub time_series_ids: Option<Vec<String>>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    pub seasonal_periods: Option<Vec<SeasonalPeriodType>>,
    /// If true, holiday_effect is a part of time series decomposition result.
    pub has_holiday_effect: Option<bool>,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    pub has_spikes_and_dips: Option<bool>,
    /// If true, step_changes is a part of time series decomposition result.
    pub has_step_changes: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DimensionalityReductionMetrics {
    /// Total percentage of variance explained by the selected principal components.
    pub total_explained_variance_ratio: Option<f64>,
}
